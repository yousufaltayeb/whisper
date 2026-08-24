import type { JsonValue } from "./protocol.generated";

export interface SettingChoice {
  value: string | boolean;
  label: string;
}

export interface SettingSpec {
  key: string;
  label: string;
  description: string;
  kind: "choice" | "integer" | "text";
  choices?: readonly SettingChoice[];
  min?: number;
  max?: number;
  suffix?: string;
  allowEmpty?: boolean;
}

export interface SettingsEditor {
  index: number;
  buffer: string;
}

const ON_OFF = [
  { value: true, label: "On" },
  { value: false, label: "Off" },
] as const;

export const SETTINGS: readonly SettingSpec[] = [
  {
    key: "language",
    label: "Language",
    description: "Automatic handles full bilingual sentences; choose English or Arabic for short utterances.",
    kind: "choice",
    choices: [
      { value: "auto", label: "Automatic" },
      { value: "en", label: "English" },
      { value: "ar", label: "Arabic" },
    ],
  },
  {
    key: "mode",
    label: "Capture mode",
    description: "Raw preserves speech, Clean normalizes it, and Code preserves developer-oriented structure.",
    kind: "choice",
    choices: [
      { value: "raw", label: "Raw" },
      { value: "clean", label: "Clean" },
      { value: "code", label: "Code" },
    ],
  },
  {
    key: "audio.backend",
    label: "Audio backend",
    description: "Automatic tries PipeWire, PulseAudio, then ALSA. Pin one only for troubleshooting.",
    kind: "choice",
    choices: [
      { value: "auto", label: "Automatic" },
      { value: "pipewire", label: "PipeWire" },
      { value: "pulse", label: "PulseAudio" },
      { value: "alsa", label: "ALSA" },
    ],
  },
  {
    key: "audio.device",
    label: "Microphone device",
    description: "Enter a device name or numeric node ID. Leave empty to use the system default.",
    kind: "text",
    allowEmpty: true,
  },
  {
    key: "audio.max_recording_seconds",
    label: "Maximum recording",
    description: "OpenWhisper cancels and deletes audio when this limit is reached.",
    kind: "integer",
    min: 10,
    max: 600,
    suffix: "seconds",
  },
  {
    key: "model.backend",
    label: "Inference backend",
    description: "Automatic prefers Vulkan and reports any CPU fallback. Vulkan never falls back silently.",
    kind: "choice",
    choices: [
      { value: "auto", label: "Automatic" },
      { value: "vulkan", label: "Vulkan" },
      { value: "cpu", label: "CPU" },
    ],
  },
  {
    key: "model.threads",
    label: "Worker threads",
    description: "Use 0 for automatic CPU thread selection.",
    kind: "integer",
    min: 0,
    max: 65_535,
    suffix: "threads",
  },
  {
    key: "delivery.clipboard",
    label: "Copy transcript",
    description: "Copy successful final transcripts to the desktop clipboard.",
    kind: "choice",
    choices: ON_OFF,
  },
  {
    key: "delivery.live_insert",
    label: "Live insertion",
    description: "Insert stable words only into the window retained at hotkey capture start. TUI recording remains preview-only.",
    kind: "choice",
    choices: ON_OFF,
  },
  {
    key: "history.enabled",
    label: "Save History",
    description: "Store successful transcripts in the private local database.",
    kind: "choice",
    choices: ON_OFF,
  },
  {
    key: "history.retention_days",
    label: "History retention",
    description: "Retention pruning runs immediately. Disable History instead of entering zero.",
    kind: "integer",
    min: 1,
    max: 65_535,
    suffix: "days",
  },
  {
    key: "privacy.local_only",
    label: "Local only",
    description: "Keep cloud providers blocked even if credentials are later configured.",
    kind: "choice",
    choices: ON_OFF,
  },
  {
    key: "notifications",
    label: "Notifications",
    description: "Allow privacy-safe desktop status notifications when an adapter is available.",
    kind: "choice",
    choices: ON_OFF,
  },
  {
    key: "overlay",
    label: "Recording overlay",
    description: "Automatic uses a supported non-focus-stealing overlay and otherwise falls back safely.",
    kind: "choice",
    choices: [
      { value: "auto", label: "Automatic" },
      { value: "always", label: "Always" },
      { value: "never", label: "Never" },
    ],
  },
  {
    key: "sounds.start",
    label: "Start sound",
    description: "Play a brief rising cue once the microphone is listening.",
    kind: "choice",
    choices: ON_OFF,
  },
  {
    key: "sounds.stop",
    label: "Stop sound",
    description: "Play a brief resolving cue after the microphone is released.",
    kind: "choice",
    choices: ON_OFF,
  },
] as const;

export function settingValue(config: Record<string, unknown>, key: string): JsonValue | undefined {
  let current: unknown = config;
  for (const segment of key.split(".")) {
    if (!current || typeof current !== "object" || Array.isArray(current)) return undefined;
    current = (current as Record<string, unknown>)[segment];
  }
  return isJsonValue(current) ? current : undefined;
}

export function displaySettingValue(spec: SettingSpec, value: JsonValue | undefined): string {
  if (spec.kind === "choice") {
    return spec.choices?.find((choice) => choice.value === value)?.label ?? String(value ?? "Unavailable");
  }
  if (spec.kind === "text") {
    const text = typeof value === "string" ? value : "";
    return text.length === 0 ? "System default" : truncate(text, 44);
  }
  const number = typeof value === "number" ? value : Number(value ?? 0);
  if (spec.key === "model.threads" && number === 0) return "Automatic (0)";
  return `${number} ${spec.suffix ?? ""}`.trim();
}

export function renderSettings(
  config: Record<string, unknown>,
  selected: number,
  editor: SettingsEditor | undefined,
  width: number,
  compact = false,
): string {
  const labelWidth = width < 72 ? 19 : 25;
  const selectedSpec = SETTINGS[selected] ?? SETTINGS[0];
  if (compact) {
    if (!selectedSpec) return "No editable settings were reported.";
    const value = editor?.index === selected
      ? `[${editor.buffer}_]`
      : displaySettingValue(selectedSpec, settingValue(config, selectedSpec.key));
    return `[${selected + 1}/${SETTINGS.length}] ${selectedSpec.label}: ${value}`;
  }
  const rows = SETTINGS.map((spec, index) => {
    const marker = index === selected ? ">" : " ";
    const value = editor?.index === index
      ? `[${editor.buffer}_]`
      : displaySettingValue(spec, settingValue(config, spec.key));
    return `${marker} ${spec.label.padEnd(labelWidth)} ${value}`;
  });
  return [
    "Settings · changes save immediately",
    "Up/Down select · Left/Right change · Enter edit/toggle",
    `${selectedSpec?.label ?? "Setting"} · ${selectedSpec?.description ?? ""}`,
    "",
    ...rows,
  ].join("\n");
}

export function cycleSetting(
  spec: SettingSpec,
  current: JsonValue | undefined,
  direction: 1 | -1,
): JsonValue | undefined {
  if (spec.kind !== "choice" || !spec.choices || spec.choices.length === 0) return undefined;
  const currentIndex = spec.choices.findIndex((choice) => choice.value === current);
  const start = currentIndex < 0 ? 0 : currentIndex;
  const next = (start + direction + spec.choices.length) % spec.choices.length;
  return spec.choices[next]?.value;
}

export function initialEditorValue(spec: SettingSpec, value: JsonValue | undefined): string {
  if (spec.kind === "text") return typeof value === "string" ? value : "";
  return typeof value === "number" ? String(value) : "";
}

export function parseEditorValue(spec: SettingSpec, input: string): JsonValue {
  if (spec.kind === "text") {
    if (/\p{Cc}/u.test(input)) throw new Error("Control characters are not allowed.");
    if (input.length > 160) throw new Error("The device value must be 160 characters or fewer.");
    const value = input.trim();
    if (!spec.allowEmpty && value.length === 0) throw new Error(`${spec.label} cannot be empty.`);
    return value;
  }
  if (spec.kind !== "integer") throw new Error(`${spec.label} is changed with Left or Right.`);
  if (!/^\d+$/.test(input)) throw new Error(`${spec.label} must be a whole number.`);
  const value = Number(input);
  if (!Number.isSafeInteger(value) || value < (spec.min ?? 0) || value > (spec.max ?? Number.MAX_SAFE_INTEGER)) {
    throw new Error(`${spec.label} must be between ${spec.min ?? 0} and ${spec.max ?? Number.MAX_SAFE_INTEGER}.`);
  }
  return value;
}

function isJsonValue(value: unknown): value is JsonValue {
  if (value === null || typeof value === "string" || typeof value === "number" || typeof value === "boolean") return true;
  if (Array.isArray(value)) return value.every(isJsonValue);
  return typeof value === "object" && Object.values(value as Record<string, unknown>).every(isJsonValue);
}

function truncate(value: string, length: number): string {
  return value.length <= length ? value : `${value.slice(0, Math.max(1, length - 1))}…`;
}
