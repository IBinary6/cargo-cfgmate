import { CargoConfig, ProfileConfig, DocConfig } from "@/types";

interface Props {
  config: CargoConfig;
  setConfig: (c: CargoConfig) => void;
  profileType: "release" | "dev";
  setProfileType: (t: "release" | "dev") => void;
}

export function BuildTab({ config, setConfig, profileType, setProfileType }: Props) {
  const currentProfile = config.profile?.[profileType] || {};

  const updateProfile = (key: string, value: any) => {
    const newProfile: ProfileConfig = { ...currentProfile };
    if (value === "" || value === undefined || value === null) {
      delete newProfile[key];
    } else {
      newProfile[key] = value;
    }
    setConfig({ ...config, profile: { ...config.profile, [profileType]: newProfile } });
  };

  // 应用预设配置
  const applyPreset = (preset: "default" | "fastest" | "smallest" | "balanced" | "fast-compile") => {
    let newProfile: ProfileConfig = {};
    
    if (preset === "default") {
      newProfile = {};
    } else if (preset === "fastest" && profileType === "release") {
      newProfile = {
        "opt-level": "3",
        lto: true,
        "codegen-units": 1,
        strip: "symbols",
      };
    } else if (preset === "smallest" && profileType === "release") {
      newProfile = {
        "opt-level": "z",
        lto: true,
        "codegen-units": 1,
        strip: "symbols",
        panic: "abort",
        "trim-paths": "all",
      };
    } else if (preset === "balanced" && profileType === "release") {
      newProfile = {
        "opt-level": "s",
        lto: "thin",
        strip: "symbols",
        panic: "abort",
        "codegen-units": 16,
      };
    } else if (preset === "fast-compile" && profileType === "dev") {
      newProfile = {
        "opt-level": "0",
      };
    }

    setConfig({ ...config, profile: { ...config.profile, [profileType]: newProfile } });
  };

  // 检测当前预设
  const detectPreset = (): string | null => {
    const profile = currentProfile;
    if (!profile || Object.keys(profile).length === 0) return "default";

    if (profileType === "release") {
      if (profile["opt-level"] === "3" && profile.lto === true &&
          profile["codegen-units"] === 1 && profile.strip === "symbols" &&
          !profile.panic && !profile["trim-paths"]) return "fastest";
      
      if (profile["opt-level"] === "z" && profile.lto === true &&
          profile["codegen-units"] === 1 && profile.strip === "symbols" &&
          profile.panic === "abort" && profile["trim-paths"] === "all") return "smallest";
      
      if (profile["opt-level"] === "s" && profile.lto === "thin" &&
          profile.strip === "symbols" && profile.panic === "abort" &&
          profile["codegen-units"] === 16 && !profile["trim-paths"]) return "balanced";
    } else if (profileType === "dev" && profile["opt-level"] === "0" && Object.keys(profile).length === 1) {
      return "fast-compile";
    }

    return null;
  };

  const currentPreset = detectPreset();

  const updateDoc = (key: string, value: any) => {
     const newDoc: DocConfig = { ...config.doc };
     if (value === "" || value === undefined || value === null) {
        delete (newDoc as any)[key];
     } else {
        (newDoc as any)[key] = value;
     }
     setConfig({ ...config, doc: newDoc });
  };

  return (
    <>
    <div className="card">
      <div className="card-header">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div className="card-title"><span style={{ color: "var(--accent-green)" }}>⚡</span> 编译优化选项</div>
          <div style={{ display: "flex", gap: 8 }}>
            <button className={`btn btn-sm ${profileType === "release" ? "btn-primary" : "btn-secondary"}`} onClick={() => setProfileType("release")}>🚀 Release</button>
            <button className={`btn btn-sm ${profileType === "dev" ? "btn-primary" : "btn-secondary"}`} onClick={() => setProfileType("dev")}>🔧 Dev</button>
          </div>
        </div>
        
        {/* 紧凑的预设配置 */}
        <div style={{ marginTop: 10, padding: "8px 10px", background: "rgba(168, 85, 247, 0.03)", border: "1px solid rgba(168, 85, 247, 0.1)", borderRadius: 6 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            <div style={{ fontSize: 12, fontWeight: 500, color: "var(--text-secondary)", display: "flex", alignItems: "center", gap: 4 }}>
              <span style={{ display: "inline-block", width: 5, height: 5, borderRadius: "50%", background: "#a855f7" }}></span>
              预设:
            </div>
            <button 
              className={`btn btn-sm ${currentPreset === "default" ? "btn-primary" : "btn-secondary"}`}
              onClick={() => applyPreset("default")}
              style={{ padding: "2px 8px", fontSize: 11 }}
            >
              ⚙️ 默认
            </button>
            {profileType === "release" && (
              <>
                <button 
                  className={`btn btn-sm ${currentPreset === "fastest" ? "btn-primary" : "btn-secondary"}`}
                  onClick={() => applyPreset("fastest")}
                  style={{ padding: "2px 8px", fontSize: 11 }}
                >
                  ⚡ 最快
                </button>
                <button 
                  className={`btn btn-sm ${currentPreset === "smallest" ? "btn-primary" : "btn-secondary"}`}
                  onClick={() => applyPreset("smallest")}
                  style={{ padding: "2px 8px", fontSize: 11 }}
                >
                  📦 最小
                </button>
                <button 
                  className={`btn btn-sm ${currentPreset === "balanced" ? "btn-primary" : "btn-secondary"}`}
                  onClick={() => applyPreset("balanced")}
                  style={{ padding: "2px 8px", fontSize: 11 }}
                >
                  ⚖️ 平衡
                </button>
              </>
            )}
            {profileType === "dev" && (
              <button 
                className={`btn btn-sm ${currentPreset === "fast-compile" ? "btn-primary" : "btn-secondary"}`}
                onClick={() => applyPreset("fast-compile")}
                style={{ padding: "2px 8px", fontSize: 11 }}
              >
                🚀 快编
              </button>
            )}
            {currentPreset && currentPreset !== "default" && (
              <span style={{ fontSize: 10, color: "var(--text-secondary)", marginLeft: "auto", fontStyle: "italic" }}>
                {currentPreset === "fastest" && "最大运行速度"}
                {currentPreset === "smallest" && "最小体积"}
                {currentPreset === "balanced" && "性能/体积平衡"}
                {currentPreset === "fast-compile" && "快速编译"}
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="card-content">
        <div className="form-row">
          <div><div className="form-label">优化等级 (opt-level)</div><div className="form-hint">数值越高优化越激进</div></div>
          <select className="select" style={{ width: 180 }} value={String(currentProfile["opt-level"] ?? "")} onChange={(e) => updateProfile("opt-level", e.target.value || undefined)}>
            <option value="">默认</option>
            <option value="0">0 - 无优化</option>
            <option value="1">1 - 基础</option>
            <option value="2">2 - 常规</option>
            <option value="3">3 - 最大</option>
            <option value="s">s - 体积优先</option>
            <option value="z">z - 最小体积</option>
          </select>
        </div>
        <div className="form-row">
          <div><div className="form-label">链接时优化 (LTO)</div><div className="form-hint">可显著减小二进制体积</div></div>
          <select
            className="select"
            style={{ width: 180 }}
            value={String(currentProfile["lto"] ?? "")}
            onChange={(e) => {
              const value = e.target.value;
              updateProfile("lto", value === "" ? undefined : value === "true" ? true : value === "false" ? false : value);
            }}
          >
            <option value="">默认</option>
            <option value="false">关闭</option>
            <option value="true">开启</option>
            <option value="thin">Thin</option>
            <option value="fat">Fat</option>
          </select>
        </div>
        <div className="form-row">
          <div><div className="form-label">Panic 处理</div><div className="form-hint">panic=abort 可减小体积</div></div>
          <select className="select" style={{ width: 180 }} value={String(currentProfile["panic"] ?? "")} onChange={(e) => updateProfile("panic", e.target.value || undefined)}>
            <option value="">默认 (unwind)</option>
            <option value="unwind">unwind</option>
            <option value="abort">abort (更小体积)</option>
          </select>
        </div>
        <div className="form-row">
          <div><div className="form-label">剥离符号 (Strip)</div><div className="form-hint">移除调试信息减小体积</div></div>
          <select className="select" style={{ width: 180 }} value={String(currentProfile["strip"] ?? "")} onChange={(e) => updateProfile("strip", e.target.value || undefined)}>
            <option value="">默认</option>
            <option value="symbols">全部剥离</option>
            <option value="none">不剥离</option>
            <option value="debuginfo">仅调试信息</option>
          </select>
        </div>
        <div className="form-row">
          <div><div className="form-label">路径裁剪 (trim-paths)</div><div className="form-hint">移除二进制中的路径信息</div></div>
          <select className="select" style={{ width: 180 }} value={String(currentProfile["trim-paths"] ?? "")} onChange={(e) => updateProfile("trim-paths", e.target.value || undefined)}>
            <option value="">默认</option>
            <option value="none">none (保留)</option>
            <option value="macro">macro</option>
            <option value="diagnostics">diagnostics</option>
            <option value="object">object</option>
            <option value="all">all (全部裁剪)</option>
          </select>
        </div>
        <div className="form-row">
          <div><div className="form-label">并发编译单元</div><div className="form-hint">设为1可最大化优化但编译慢</div></div>
          <input
            type="number"
            className="input"
            style={{ width: 180 }}
            placeholder="默认"
            min="1"
            max="256"
            value={currentProfile["codegen-units"] ?? ""}
            onChange={(e) => {
              const val = e.target.value ? parseInt(e.target.value) : undefined;
              if (val !== undefined && (val <= 0 || val > 256)) return;
              updateProfile("codegen-units", val);
            }}
          />
        </div>
      </div>
    </div>

    {/* 文档配置卡片 */}
    <div className="card">
      <div className="card-header">
        <div className="card-title"><span style={{ color: "var(--accent-blue)" }}>📚</span> 文档生成 (Cargo Doc)</div>
      </div>
      <div className="card-content">
        <div className="form-row">
          <div>
             <div className="form-label">默认浏览器</div>
             <div className="form-hint">`cargo doc --open` 使用的程序</div>
          </div>
          <input 
            className="input" 
            placeholder="例如: chromium" 
            style={{ width: 180 }}
            value={config.doc?.browser || ""}
            onChange={(e) => updateDoc("browser", e.target.value)}
          />
        </div>
        <div className="form-row">
          <div>
            <div className="form-label">自动打开 (open-result)</div>
            <div className="form-hint">每次生成文档自动打开浏览器</div>
          </div>
           <select 
             className="select" 
             style={{ width: 180 }}
             value={config.doc?.["open-result"] === undefined ? "" : String(config.doc["open-result"])}
             onChange={(e) => {
               const val = e.target.value;
               updateDoc("open-result", val === "" ? undefined : val === "true");
             }}
           >
             <option value="">默认 (false)</option>
             <option value="true">开启 (True)</option>
             <option value="false">关闭 (False)</option>
           </select>
        </div>
      </div>
    </div>
    </>
  );
}
