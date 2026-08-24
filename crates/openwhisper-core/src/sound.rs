//! Short, dependency-free capture earcons and best-effort native playback.
//!
//! Cues are synthesized in memory so release packages do not need remote or
//! separately installed media assets. Playback failure must never affect the
//! privacy-sensitive capture state machine.

use std::f32::consts::TAU;
use std::io;
use std::process::Stdio;
use std::time::Duration;

use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const SAMPLE_RATE: u32 = 48_000;
const PLAYBACK_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundCue {
    ListeningStarted,
    ListeningStopped,
}

impl SoundCue {
    pub const fn label(self) -> &'static str {
        match self {
            Self::ListeningStarted => "listening started",
            Self::ListeningStopped => "listening stopped",
        }
    }
}

#[derive(Debug, Error)]
pub enum SoundError {
    #[error("no supported sound player is available")]
    PlayerUnavailable,
    #[error("sound playback failed: {0}")]
    Playback(#[from] io::Error),
    #[error("sound player did not accept the cue")]
    Rejected,
    #[error("sound playback timed out")]
    Timeout,
}

/// Play one capture cue without touching capture input or persisted user data.
///
/// The caller should run this in a detached task. The function is intentionally
/// best effort: callers report failures only at debug level and never turn an
/// unavailable desktop player into a recording failure.
pub async fn play_sound_cue(cue: SoundCue) -> Result<(), SoundError> {
    let pcm = render_cue(cue);

    #[cfg(target_os = "linux")]
    {
        return play_linux(&pcm, cue).await;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pcm;
        Err(SoundError::PlayerUnavailable)
    }
}

#[cfg(target_os = "linux")]
async fn play_linux(pcm: &[u8], cue: SoundCue) -> Result<(), SoundError> {
    let stream_name = match cue {
        SoundCue::ListeningStarted => "OpenWhisper listening started",
        SoundCue::ListeningStopped => "OpenWhisper listening stopped",
    };
    let players = [
        Player {
            program: "pw-play",
            args: &[
                "--media-category",
                "Playback",
                "--media-role",
                "Notification",
                "--latency",
                "40ms",
                "--rate",
                "48000",
                "--channels",
                "1",
                "--format",
                "s16",
                "--raw",
                "-",
            ],
        },
        Player {
            program: "paplay",
            args: &[
                "--raw",
                "--rate=48000",
                "--format=s16le",
                "--channels=1",
                "--client-name=OpenWhisper",
                "-",
            ],
        },
        Player {
            program: "aplay",
            args: &[
                "-q", "-t", "raw", "-f", "S16_LE", "-r", "48000", "-c", "1", "-",
            ],
        },
    ];

    let mut found_player = false;
    let mut last_error = None;
    let mut timed_out = false;
    for player in players {
        let mut command = Command::new(player.program);
        if player.program == "pw-play" {
            command.arg("-P").arg(format!("media.name={stream_name}"));
        } else if player.program == "paplay" {
            command.arg(format!("--stream-name={stream_name}"));
        }
        command
            .args(player.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);

        let mut child = match command.spawn() {
            Ok(child) => {
                found_player = true;
                child
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                found_player = true;
                last_error = Some(error);
                continue;
            }
        };
        let Some(mut input) = child.stdin.take() else {
            last_error = Some(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "sound player stdin was unavailable",
            ));
            continue;
        };
        if let Err(error) = input.write_all(pcm).await {
            last_error = Some(error);
            continue;
        }
        drop(input);

        match tokio::time::timeout(PLAYBACK_TIMEOUT, child.wait()).await {
            Ok(Ok(status)) if status.success() => return Ok(()),
            Ok(Ok(_)) => {}
            Ok(Err(error)) => last_error = Some(error),
            Err(_) => {
                let _ = child.kill().await;
                timed_out = true;
            }
        }
    }

    match (found_player, last_error, timed_out) {
        (false, _, _) => Err(SoundError::PlayerUnavailable),
        (_, Some(error), _) => Err(SoundError::Playback(error)),
        (_, None, true) => Err(SoundError::Timeout),
        _ => Err(SoundError::Rejected),
    }
}

#[cfg(target_os = "linux")]
struct Player {
    program: &'static str,
    args: &'static [&'static str],
}

fn render_cue(cue: SoundCue) -> Vec<u8> {
    let duration = match cue {
        SoundCue::ListeningStarted => 0.155,
        SoundCue::ListeningStopped => 0.175,
    };
    let sample_count = (SAMPLE_RATE as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut primary_phase = 0.0_f32;
    let mut secondary_phase = 0.0_f32;
    let mut body_phase = 0.0_f32;

    for index in 0..sample_count {
        let time = index as f32 / SAMPLE_RATE as f32;
        let progress = time / duration;
        let curved = smoothstep(progress);
        let (primary_frequency, secondary_frequency, body_frequency, secondary_start) = match cue {
            SoundCue::ListeningStarted => (
                mix(520.0, 690.0, curved),
                mix(760.0, 980.0, curved),
                mix(330.0, 390.0, curved),
                0.032,
            ),
            SoundCue::ListeningStopped => (
                mix(820.0, 560.0, curved),
                mix(620.0, 430.0, curved),
                mix(390.0, 320.0, curved),
                0.026,
            ),
        };
        primary_phase += TAU * primary_frequency / SAMPLE_RATE as f32;
        secondary_phase += TAU * secondary_frequency / SAMPLE_RATE as f32;
        body_phase += TAU * body_frequency / SAMPLE_RATE as f32;

        let main_envelope = envelope(time, duration, 0.006, 0.065);
        let secondary_time = (time - secondary_start).max(0.0);
        let secondary_duration = duration - secondary_start;
        let secondary_envelope = if time >= secondary_start {
            envelope(secondary_time, secondary_duration, 0.005, 0.055)
        } else {
            0.0
        };
        let body_envelope = envelope(time, duration, 0.010, 0.080);

        // Slightly inharmonic upper partials keep the cue closer to a soft
        // studio control than a plain electronic beep.
        let primary = timbre(primary_phase) * main_envelope * 0.58;
        let secondary = timbre(secondary_phase) * secondary_envelope * 0.31;
        let body = body_phase.sin() * body_envelope * 0.11;
        samples.push(primary + secondary + body);
    }

    normalize_pcm(
        samples,
        match cue {
            SoundCue::ListeningStarted => 0.17,
            SoundCue::ListeningStopped => 0.15,
        },
    )
}

fn timbre(phase: f32) -> f32 {
    phase.sin() * 0.84 + (phase * 2.01 + 0.2).sin() * 0.12 + (phase * 3.97).sin() * 0.04
}

fn envelope(time: f32, duration: f32, attack: f32, release: f32) -> f32 {
    let fade_in = smoothstep((time / attack).clamp(0.0, 1.0));
    let fade_out = smoothstep(((duration - time) / release).clamp(0.0, 1.0));
    fade_in * fade_out
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn mix(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

fn normalize_pcm(samples: Vec<f32>, target_peak: f32) -> Vec<u8> {
    let peak = samples
        .iter()
        .fold(0.0_f32, |current, sample| current.max(sample.abs()));
    let scale = if peak > 0.0 { target_peak / peak } else { 0.0 };
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let value = (sample * scale * i16::MAX as f32).round() as i16;
        pcm.extend_from_slice(&value.to_le_bytes());
    }
    pcm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cues_are_brief_distinct_and_gently_limited() {
        let start = render_cue(SoundCue::ListeningStarted);
        let stop = render_cue(SoundCue::ListeningStopped);

        assert_eq!(start.len(), (SAMPLE_RATE as f32 * 0.155) as usize * 2);
        assert_eq!(stop.len(), (SAMPLE_RATE as f32 * 0.175) as usize * 2);
        assert_ne!(start, stop);
        assert_eq!(&start[..2], &[0, 0]);
        assert_eq!(&stop[..2], &[0, 0]);

        let start_peak = peak(&start);
        let stop_peak = peak(&stop);
        assert!((5_500..=5_600).contains(&start_peak));
        assert!((4_900..=5_000).contains(&stop_peak));
    }

    fn peak(pcm: &[u8]) -> i16 {
        pcm.chunks_exact(2)
            .map(|sample| i16::from_le_bytes([sample[0], sample[1]]).abs())
            .max()
            .unwrap_or_default()
    }
}
