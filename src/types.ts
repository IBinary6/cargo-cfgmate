export interface SourceEntry {
    registry?: string;
    "replace-with"?: string;
    [key: string]: any;
}

export interface RegistryConfig {
    default?: string;
}

export interface NetConfig {
    offline?: boolean;
    "git-fetch-with-cli"?: boolean;
}

export interface HttpConfig {
    proxy?: string;
    "check-revoke"?: boolean;
    multiplexing?: boolean;
    cainfo?: string;
    timeout?: number;
    "low-speed-limit"?: number;
}

export interface HttpsConfig {
    proxy?: string;
}

export interface DocConfig {
    browser?: string;
    "open-result"?: boolean;
}

export interface RegistryEntry {
    index?: string;
    token?: string;
}

export interface BuildConfig {
    jobs?: number;
    target?: string;
    "rustc-wrapper"?: string;
    rustflags?: string[];
    rustdocflags?: string[];
    [key: string]: any;
}

export interface TargetConfig {
    linker?: string;
    ar?: string;
    rustflags?: string[];
    runner?: string;
    [key: string]: any;
}

export interface ProfileConfig {
    "opt-level"?: string | number;
    lto?: string | boolean;
    "codegen-units"?: number;
    debug?: boolean | number;
    strip?: boolean | string;
    panic?: string;
    "trim-paths"?: string;
    [key: string]: any;
}

export interface EnvObject {
    value: string;
    force?: boolean;
    relative?: boolean;
}

export interface CargoConfig {
    source?: Record<string, SourceEntry>;
    registry?: RegistryConfig;
    net?: NetConfig;
    http?: HttpConfig;
    https?: HttpsConfig;
    build?: BuildConfig;
    target?: Record<string, TargetConfig>;
    env?: Record<string, string | EnvObject>;
    profile?: Record<string, ProfileConfig>;
    alias?: Record<string, string>;
    doc?: DocConfig;
    registries?: Record<string, RegistryEntry>;
    [key: string]: any;
}

export interface RustupEnvScopeStatus {
    value?: string;
    source?: string;
    error?: string;
}

export interface RustupEnvVarStatus {
    user: RustupEnvScopeStatus;
    system: RustupEnvScopeStatus;
}

export interface RustupEnvStatus {
    dist: RustupEnvVarStatus;
    root: RustupEnvVarStatus;
}

export interface RustupEnvWriteOutcome {
    ok: boolean;
    error?: string;
    skipped?: boolean;
}

export interface RustupEnvWriteResult {
    user: RustupEnvWriteOutcome;
    system: RustupEnvWriteOutcome;
}

export interface AdminStatus {
    is_admin: boolean;
    hint: string;
}

export interface BackupEntry {
    name: string;
    path: string;
    modified: number;
    size: number;
}

// 常用目标平台
export const TARGET_PLATFORMS = [
    { value: "", label: "默认 (当前系统)" },
    { value: "x86_64-pc-windows-msvc", label: "Windows x64 (MSVC)" },
    { value: "x86_64-pc-windows-gnu", label: "Windows x64 (GNU/MinGW)" },
    { value: "i686-pc-windows-msvc", label: "Windows x86 (MSVC)" },
    { value: "x86_64-unknown-linux-gnu", label: "Linux x64 (GNU)" },
    { value: "x86_64-unknown-linux-musl", label: "Linux x64 (musl静态)" },
    { value: "aarch64-unknown-linux-gnu", label: "Linux ARM64" },
    { value: "x86_64-apple-darwin", label: "macOS x64" },
    { value: "aarch64-apple-darwin", label: "macOS ARM64 (M1/M2)" },
    { value: "wasm32-unknown-unknown", label: "WebAssembly" },
];

// 链接器选项
export const LINKER_OPTIONS = [
    { value: "", label: "默认链接器" },
    { value: "lld-link", label: "lld-link (LLVM, 推荐Windows)" },
    { value: "rust-lld", label: "rust-lld (Rust内置)" },
    { value: "mold", label: "mold (超快, Linux)" },
    { value: "gold", label: "gold (GNU Gold, Linux)" },
    { value: "link.exe", label: "link.exe (MSVC)" },
];

// 编译缓存选项
export const WRAPPER_OPTIONS = [
    { value: "", label: "无 (不使用缓存)" },
    { value: "sccache", label: "sccache (推荐)" },
    { value: "ccache", label: "ccache" },
];

// 常用 Rustflags
export const COMMON_RUSTFLAGS = [
    { value: "-Ctarget-feature=+crt-static", label: "Static CRT", desc: "静态链接运行时(Windows)" },
    { value: "-Cprefer-dynamic", label: "Prefer Dynamic", desc: "优先动态链接" },
    { value: "-Clink-arg=-s", label: "Strip Symbols", desc: "剥离符号(减小体积)" },
    { value: "-Ctarget-cpu=native", label: "Native CPU", desc: "针对当前CPU优化" },
];

// 代理预设
export const PROXY_PRESETS = [
  { label: "无代理", value: "" },
  { label: "Clash (7890)", value: "127.0.0.1:7890" },
  { label: "Clash (7891)", value: "127.0.0.1:7891" },
  { label: "Clash Rev (7897)", value: "127.0.0.1:7897" },
  { label: "V2Ray (10808)", value: "127.0.0.1:10808" },
  { label: "V2Ray (1080)", value: "127.0.0.1:1080" },
  { label: "自定义...", value: "custom" },
];
