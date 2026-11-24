//! UI translations.
//!
//! Every language is one `Strings` value, so a missing translation is a
//! compile error rather than a blank label. Messages that come from the
//! backend (errors, release notes, platform names) and log lines are shown as
//! received and are not translated here.

use leptos::prelude::*;

const STORAGE_KEY: &str = "rustfs_lang";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
    Fr,
    Ja,
}

impl Lang {
    pub const ALL: [Lang; 4] = [Lang::Zh, Lang::En, Lang::Fr, Lang::Ja];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Zh => "zh-CN",
            Lang::Fr => "fr",
            Lang::Ja => "ja",
        }
    }

    /// Short label on the switcher, written in the language itself.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Zh => "中",
            Lang::Fr => "FR",
            Lang::Ja => "日",
        }
    }

    /// Full name for the switcher tooltip and screen readers.
    pub fn full_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Zh => "简体中文",
            Lang::Fr => "Français",
            Lang::Ja => "日本語",
        }
    }

    pub fn strings(self) -> &'static Strings {
        match self {
            Lang::En => &EN,
            Lang::Zh => &ZH,
            Lang::Fr => &FR,
            Lang::Ja => &JA,
        }
    }

    /// Matches on the primary subtag, so `fr-CA` and `ja_JP` are recognised.
    fn from_tag(tag: &str) -> Option<Lang> {
        let primary = tag
            .trim()
            .split(['-', '_'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        match primary.as_str() {
            "zh" => Some(Lang::Zh),
            "en" => Some(Lang::En),
            "fr" => Some(Lang::Fr),
            "ja" => Some(Lang::Ja),
            _ => None,
        }
    }
}

/// A language the user picked wins over the system language; anything the
/// launcher has no translation for falls back to English.
pub fn resolve_lang(saved: Option<&str>, system: Option<&str>) -> Lang {
    saved
        .and_then(Lang::from_tag)
        .or_else(|| system.and_then(Lang::from_tag))
        .unwrap_or(Lang::En)
}

fn saved_lang() -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(STORAGE_KEY)
        .ok()?
}

fn system_lang() -> Option<String> {
    web_sys::window()?.navigator().language()
}

fn apply_document_lang(lang: Lang) {
    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
    {
        let _ = root.set_attribute("lang", lang.code());
    }
}

#[derive(Clone, Copy)]
pub struct I18n {
    lang: RwSignal<Lang>,
}

impl I18n {
    pub fn lang(self) -> Lang {
        self.lang.get()
    }

    /// Strings for the current language; tracks the signal so views re-render
    /// when the language changes.
    pub fn t(self) -> &'static Strings {
        self.lang.get().strings()
    }

    /// Strings for event handlers and async tasks that must not subscribe.
    pub fn t_untracked(self) -> &'static Strings {
        self.lang.get_untracked().strings()
    }

    pub fn set(self, lang: Lang) {
        self.lang.set(lang);
        apply_document_lang(lang);
        if let Some(Ok(Some(storage))) = web_sys::window().map(|window| window.local_storage()) {
            let _ = storage.set_item(STORAGE_KEY, lang.code());
        }
    }
}

pub fn provide_i18n() -> I18n {
    let lang = resolve_lang(saved_lang().as_deref(), system_lang().as_deref());
    apply_document_lang(lang);
    let i18n = I18n {
        lang: RwSignal::new(lang),
    };
    provide_context(i18n);
    i18n
}

pub fn use_i18n() -> I18n {
    expect_context::<I18n>()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionLabel {
    Launching,
    Stop,
    Launch,
}

pub struct Strings {
    // Banner and language switcher
    pub banner_text: &'static str,
    pub banner_close: &'static str,
    pub language: &'static str,

    // Header and status
    pub service_online: &'static str,
    pub service_offline: &'static str,
    pub settings: &'static str,
    pub close_settings: &'static str,
    pub settings_note: &'static str,

    // Pre-start form and overlays
    pub select_data_dir: &'static str,
    pub data_path: &'static str,
    pub data_path_placeholder: &'static str,
    pub browse: &'static str,
    pub host: &'static str,
    pub api_port: &'static str,
    pub console: &'static str,
    pub console_port: &'static str,
    pub change_admin: &'static str,
    pub close_change_admin: &'static str,
    pub admin_note: &'static str,
    pub access_key: &'static str,
    pub secret_key: &'static str,
    pub launching: &'static str,
    pub stop_rustfs: &'static str,
    pub launch_rustfs: &'static str,
    pub external_listener: &'static str,
    pub api_port_invalid: &'static str,
    pub console_port_invalid: &'static str,
    pub port_conflict: &'static str,
    pub data_path_required: &'static str,

    // Post-start endpoint and secondary actions
    pub endpoint_region: &'static str,
    pub reachability_lan: &'static str,
    pub reachability_local: &'static str,
    pub listening: &'static str,
    pub open_in_browser: &'static str,
    pub copy: &'static str,
    pub copied: &'static str,
    pub copy_failed: &'static str,
    pub open_console: &'static str,
    pub view_logs: &'static str,
    pub hide_logs: &'static str,
    pub logs_region: &'static str,

    // Footer and upgrade overlay
    pub current_version: fn(&str) -> String,
    pub current_version_with_rustfs: fn(&str, &str) -> String,
    pub product_links: &'static str,
    pub upgrade: &'static str,
    pub close_upgrade: &'static str,
    pub website: &'static str,
    pub docs: &'static str,
    pub github: &'static str,
    pub checking: &'static str,
    pub check_for_updates: &'static str,
    pub unknown: &'static str,
    pub source_installed: &'static str,
    pub source_bundled: &'static str,
    pub launcher_version: fn(&str) -> String,
    pub rustfs_version: fn(&str, &str) -> String,
    pub no_release_notes: &'static str,
    pub installing: &'static str,
    pub update_launcher: &'static str,
    pub update_rustfs: &'static str,
    pub launcher_up_to_date: &'static str,
    pub rustfs_up_to_date: &'static str,
    pub verified_download: &'static str,
    pub launcher_progress: &'static str,
    pub rustfs_progress: &'static str,

    // Toasts and confirmations
    pub launching_rustfs: &'static str,
    pub launch_succeeded: &'static str,
    pub stopping_rustfs: &'static str,
    pub rustfs_stopped: &'static str,
    pub all_up_to_date: &'static str,
    pub update_available: fn(&str) -> String,
    pub confirm_launcher_update: &'static str,
    pub confirm_rustfs_update: &'static str,
    pub installing_launcher_update: &'static str,
    pub installing_rustfs_update: &'static str,
    pub dismiss_notification: &'static str,

    // First-screen failure reasons the launcher writes itself
    pub reason_launch_default: &'static str,
    pub reason_stop_default: &'static str,
    pub reason_upgrade_default: &'static str,
    pub reason_check_default: &'static str,
    pub reason_already_running: &'static str,
    pub reason_no_update: &'static str,
    pub reason_not_connected: &'static str,
    pub reason_unreadable_result: &'static str,

    // First-screen fixes
    pub fix_launch_default: &'static str,
    pub fix_stop_default: &'static str,
    pub fix_upgrade_default: &'static str,
    pub fix_check_default: &'static str,
    pub fix_choose_data_folder: &'static str,
    pub fix_create_folder: &'static str,
    pub fix_folder_not_file: &'static str,
    pub fix_pick_local_folder: &'static str,
    pub fix_writable_parent: &'static str,
    pub fix_port_in_use: &'static str,
    pub fix_different_ports: &'static str,
    pub fix_port_number: &'static str,
    pub fix_bind_host: &'static str,
    pub fix_public_credentials: &'static str,
    pub fix_already_running: &'static str,
    pub fix_binary_not_found: &'static str,
    pub fix_binary_exec: &'static str,
    pub fix_launch_exit: &'static str,
    pub fix_retry_rustfs_update: &'static str,
    pub fix_network_update: &'static str,
    pub fix_no_update: &'static str,
    pub fix_asset_missing: &'static str,
    pub fix_open_from_shell: &'static str,

    // Log viewer
    pub log_sources: &'static str,
    pub app_logs: &'static str,
    pub rustfs_output: &'static str,
    pub auto_scroll: &'static str,
    pub clear_logs: &'static str,
    pub clear: &'static str,
    pub no_logs: &'static str,
}

impl Strings {
    pub fn action_label(&self, label: ActionLabel) -> &'static str {
        match label {
            ActionLabel::Launching => self.launching,
            ActionLabel::Stop => self.stop_rustfs,
            ActionLabel::Launch => self.launch_rustfs,
        }
    }
}

pub const EN: Strings = Strings {
    banner_text: "The desktop edition is not recommended for production use",
    banner_close: "Dismiss notice",
    language: "Language",

    service_online: "Service online",
    service_offline: "Service offline",
    settings: "Settings",
    close_settings: "Close settings",
    settings_note: "Host, port, and console apply the next time you launch. A new install fills in 0.0.0.0, so other devices on your network can connect. An empty host stays on this computer.",

    select_data_dir: "Select RustFS Data Directory",
    data_path: "Data path",
    data_path_placeholder: "Select or drop a data folder",
    browse: "Browse",
    host: "Host",
    api_port: "API port",
    console: "Console",
    console_port: "Console port",
    change_admin: "Change administrator",
    close_change_admin: "Close change administrator",
    admin_note: "These keys are used the next time you launch. RustFS keeps the keys shown on the main screen until then. The published default is rustfsadmin.",
    access_key: "Access key",
    secret_key: "Secret key",
    launching: "Launching…",
    stop_rustfs: "Stop RustFS",
    launch_rustfs: "Launch RustFS",
    external_listener: "Something is already listening on this host and port, but this launcher did not start it. Change the port in Settings, or stop that program, then launch.",
    api_port_invalid: "API port must be a number between 1 and 65535.",
    console_port_invalid: "Console port must be a number between 1 and 65535.",
    port_conflict: "API port and console port must be different.",
    data_path_required: "Data path is required.",

    endpoint_region: "Listening address and credentials",
    reachability_lan: "Other devices on your network can reach this endpoint. The keys below are the ones RustFS is using.",
    reachability_local: "Only this computer can reach this endpoint. The keys below are the ones RustFS is using.",
    listening: "Listening",
    open_in_browser: "Open in browser",
    copy: "Copy",
    copied: "Copied",
    copy_failed: "Could not copy. Select the value and copy it manually.",
    open_console: "Open Console",
    view_logs: "View logs",
    hide_logs: "Hide logs",
    logs_region: "Logs",

    current_version: |version| format!("Current version: {version}"),
    current_version_with_rustfs: |launcher, rustfs| {
        format!("Current version: {launcher} · RustFS {rustfs}")
    },
    product_links: "Product links",
    upgrade: "Upgrade",
    close_upgrade: "Close upgrade",
    website: "Website",
    docs: "Docs",
    github: "GitHub",
    checking: "Checking…",
    check_for_updates: "Check for Updates",
    unknown: "unknown",
    source_installed: "installed",
    source_bundled: "bundled",
    launcher_version: |version| format!("Launcher {version}"),
    rustfs_version: |version, source| format!("RustFS {version} ({source})"),
    no_release_notes: "No release notes for this version.",
    installing: "Installing…",
    update_launcher: "Update Launcher",
    update_rustfs: "Update RustFS",
    launcher_up_to_date: "Launcher is up to date.",
    rustfs_up_to_date: "RustFS is up to date.",
    verified_download: "Verified download from the RustFS release feed.",
    launcher_progress: "Launcher update progress",
    rustfs_progress: "RustFS update progress",

    launching_rustfs: "Launching RustFS...",
    launch_succeeded: "RustFS launched successfully!",
    stopping_rustfs: "Stopping RustFS...",
    rustfs_stopped: "RustFS stopped",
    all_up_to_date: "You are running the latest versions",
    update_available: |found| format!("Update available: {found}"),
    confirm_launcher_update: "Updating the launcher stops the RustFS service it manages and restarts the app once the update is installed. Continue?",
    confirm_rustfs_update: "Updating RustFS stops the service managed by this launcher. Launch it again afterwards to use the new build. Continue?",
    installing_launcher_update: "Downloading and installing the launcher update…",
    installing_rustfs_update: "Downloading and installing the RustFS update…",
    dismiss_notification: "Dismiss notification",

    reason_launch_default: "RustFS did not start.",
    reason_stop_default: "RustFS did not stop.",
    reason_upgrade_default: "The update did not finish.",
    reason_check_default: "The update check did not finish.",
    reason_already_running: "RustFS is already running.",
    reason_no_update: "No update is available.",
    reason_not_connected: "The window is not connected to the desktop shell.",
    reason_unreadable_result: "The launcher could not read the result of the action.",

    fix_launch_default: "Check the data path and ports, then launch again.",
    fix_stop_default: "Press Stop RustFS again. If the service is still running, quit the launcher from the tray menu.",
    fix_upgrade_default: "Try the update again. A failed update leaves the installed build unchanged.",
    fix_check_default: "Check the network connection, then press Check for Updates again.",
    fix_choose_data_folder: "Choose a folder with Browse, or type the path, then launch again.",
    fix_create_folder: "Create this folder, then launch again. The launcher does not create it for you.",
    fix_folder_not_file: "Choose a folder, not a file, then launch again.",
    fix_pick_local_folder: "Pick a local folder with Browse, then launch again.",
    fix_writable_parent: "Choose a data folder whose parent directory is writable, then launch again.",
    fix_port_in_use: "Choose a different port, or stop the program that is listening on it, then launch again.",
    fix_different_ports: "Set a different API port and console port, then launch again.",
    fix_port_number: "Enter a whole number from 1 to 65535, then launch again.",
    fix_bind_host: "Use 127.0.0.1, localhost, or an IP address such as 0.0.0.0, then launch again.",
    fix_public_credentials: "Bind to 127.0.0.1, or set a unique access key and secret key. Empty credentials also fall back to rustfsadmin.",
    fix_already_running: "Press Stop RustFS if this launcher started it. Otherwise change the port, or stop the other program, then launch again.",
    fix_binary_not_found: "Reinstall RustFS Launcher, or install the RustFS build from Check for Updates, then launch again.",
    fix_binary_exec: "Reinstall RustFS Launcher so the bundled binary is restored, then launch again.",
    fix_launch_exit: "Confirm the data folder exists and the port is free, then launch again. RustFS writes more detail in the logs folder next to the data directory.",
    fix_retry_rustfs_update: "Run the RustFS update again. The current install was left unchanged.",
    fix_network_update: "Check the network connection, then try again. A failed update leaves the installed build unchanged.",
    fix_no_update: "This launcher is already current. Launch RustFS with the installed build.",
    fix_asset_missing: "Try again later. The installed build is unchanged.",
    fix_open_from_shell: "Open RustFS Launcher from the Start menu or the Applications folder, then launch again.",

    log_sources: "Log sources",
    app_logs: "App Logs",
    rustfs_output: "RustFS Output",
    auto_scroll: "Auto-scroll",
    clear_logs: "Clear Logs",
    clear: "Clear",
    no_logs: "No logs available",
};

pub const ZH: Strings = Strings {
    banner_text: "桌面版本不建议在生产使用",
    banner_close: "关闭提示",
    language: "语言",

    service_online: "服务在线",
    service_offline: "服务离线",
    settings: "设置",
    close_settings: "关闭设置",
    settings_note: "监听地址、端口和控制台会在下次启动时生效。全新安装默认填写 0.0.0.0，局域网内的其他设备也可连接；留空则仅在本机监听。",

    select_data_dir: "选择 RustFS 数据目录",
    data_path: "数据路径",
    data_path_placeholder: "选择或拖入数据文件夹",
    browse: "浏览",
    host: "监听地址",
    api_port: "API 端口",
    console: "控制台",
    console_port: "控制台端口",
    change_admin: "更改管理员",
    close_change_admin: "关闭更改管理员",
    admin_note: "这些密钥会在下次启动时生效，在此之前 RustFS 仍使用主界面显示的密钥。公开默认值为 rustfsadmin。",
    access_key: "Access Key",
    secret_key: "Secret Key",
    launching: "启动中…",
    stop_rustfs: "停止 RustFS",
    launch_rustfs: "启动 RustFS",
    external_listener:
        "该地址和端口已被其他程序监听，但不是由本启动器启动的。请在“设置”中更换端口，或停止该程序后再启动。",
    api_port_invalid: "API 端口必须是 1 到 65535 之间的数字。",
    console_port_invalid: "控制台端口必须是 1 到 65535 之间的数字。",
    port_conflict: "API 端口和控制台端口不能相同。",
    data_path_required: "请填写数据路径。",

    endpoint_region: "监听地址与凭证",
    reachability_lan: "同一网络中的其他设备可以访问此地址。下方显示的是 RustFS 当前使用的密钥。",
    reachability_local: "仅本机可以访问此地址。下方显示的是 RustFS 当前使用的密钥。",
    listening: "监听地址",
    open_in_browser: "在浏览器中打开",
    copy: "复制",
    copied: "已复制",
    copy_failed: "无法复制。请手动选中内容并复制。",
    open_console: "打开控制台",
    view_logs: "查看日志",
    hide_logs: "隐藏日志",
    logs_region: "日志",

    current_version: |version| format!("当前版本：{version}"),
    current_version_with_rustfs: |launcher, rustfs| {
        format!("当前版本：{launcher} · RustFS {rustfs}")
    },
    product_links: "产品链接",
    upgrade: "升级",
    close_upgrade: "关闭升级",
    website: "官网",
    docs: "文档",
    github: "GitHub",
    checking: "检查中…",
    check_for_updates: "检查更新",
    unknown: "未知",
    source_installed: "应用内安装",
    source_bundled: "内置",
    launcher_version: |version| format!("启动器 {version}"),
    rustfs_version: |version, source| format!("RustFS {version}（{source}）"),
    no_release_notes: "此版本没有更新说明。",
    installing: "安装中…",
    update_launcher: "更新启动器",
    update_rustfs: "更新 RustFS",
    launcher_up_to_date: "启动器已是最新版本。",
    rustfs_up_to_date: "RustFS 已是最新版本。",
    verified_download: "从 RustFS 官方发布源下载并校验。",
    launcher_progress: "启动器更新进度",
    rustfs_progress: "RustFS 更新进度",

    launching_rustfs: "正在启动 RustFS…",
    launch_succeeded: "RustFS 启动成功！",
    stopping_rustfs: "正在停止 RustFS…",
    rustfs_stopped: "RustFS 已停止",
    all_up_to_date: "已是最新版本",
    update_available: |found| format!("有可用更新：{found}"),
    confirm_launcher_update:
        "更新启动器会停止它所管理的 RustFS 服务，并在安装完成后重启应用。是否继续？",
    confirm_rustfs_update:
        "更新 RustFS 会停止由本启动器管理的服务，更新后需要重新启动才能使用新版本。是否继续？",
    installing_launcher_update: "正在下载并安装启动器更新…",
    installing_rustfs_update: "正在下载并安装 RustFS 更新…",
    dismiss_notification: "关闭通知",

    reason_launch_default: "RustFS 未能启动。",
    reason_stop_default: "RustFS 未能停止。",
    reason_upgrade_default: "更新未完成。",
    reason_check_default: "更新检查未完成。",
    reason_already_running: "RustFS 已在运行。",
    reason_no_update: "没有可用的更新。",
    reason_not_connected: "窗口没有连接到桌面程序。",
    reason_unreadable_result: "启动器无法读取操作结果。",

    fix_launch_default: "请检查数据路径和端口，然后重新启动。",
    fix_stop_default: "请再次点击“停止 RustFS”。如果服务仍在运行，请从托盘菜单退出启动器。",
    fix_upgrade_default: "请重新尝试更新。更新失败不会改动已安装的版本。",
    fix_check_default: "请检查网络连接，然后再次点击“检查更新”。",
    fix_choose_data_folder: "请点击“浏览”选择文件夹，或直接输入路径，然后重新启动。",
    fix_create_folder: "请先创建这个文件夹，然后重新启动。启动器不会自动创建。",
    fix_folder_not_file: "请选择文件夹而不是文件，然后重新启动。",
    fix_pick_local_folder: "请点击“浏览”选择一个本地文件夹，然后重新启动。",
    fix_writable_parent: "请选择上级目录可写的数据文件夹，然后重新启动。",
    fix_port_in_use: "请换一个端口，或停止正在监听该端口的程序，然后重新启动。",
    fix_different_ports: "请为 API 端口和控制台端口设置不同的值，然后重新启动。",
    fix_port_number: "请输入 1 到 65535 之间的整数，然后重新启动。",
    fix_bind_host: "请使用 127.0.0.1、localhost 或 0.0.0.0 这样的 IP 地址，然后重新启动。",
    fix_public_credentials: "请绑定到 127.0.0.1，或设置独立的 Access Key 和 Secret Key。留空同样会回退为 rustfsadmin。",
    fix_already_running: "如果是本启动器启动的，请点击“停止 RustFS”；否则请更换端口或停止另一个程序，然后重新启动。",
    fix_binary_not_found: "请重新安装 RustFS Launcher，或通过“检查更新”安装 RustFS，然后重新启动。",
    fix_binary_exec: "请重新安装 RustFS Launcher 以恢复内置的二进制文件，然后重新启动。",
    fix_launch_exit: "请确认数据文件夹存在且端口未被占用，然后重新启动。RustFS 会把详细信息写入数据目录旁边的 logs 文件夹。",
    fix_retry_rustfs_update: "请重新执行 RustFS 更新。当前安装的版本没有被改动。",
    fix_network_update: "请检查网络连接后重试。更新失败不会改动已安装的版本。",
    fix_no_update: "启动器已是最新版本，请直接用已安装的版本启动 RustFS。",
    fix_asset_missing: "请稍后再试。已安装的版本没有被改动。",
    fix_open_from_shell: "请从开始菜单或“应用程序”文件夹打开 RustFS Launcher，然后重新启动。",

    log_sources: "日志来源",
    app_logs: "应用日志",
    rustfs_output: "RustFS 输出",
    auto_scroll: "自动滚动",
    clear_logs: "清空日志",
    clear: "清空",
    no_logs: "暂无日志",
};

pub const FR: Strings = Strings {
    banner_text: "La version de bureau n'est pas recommandée en production",
    banner_close: "Fermer l'avertissement",
    language: "Langue",

    service_online: "Service en ligne",
    service_offline: "Service hors ligne",
    settings: "Paramètres",
    close_settings: "Fermer les paramètres",
    settings_note: "L'hôte, le port et la console s'appliquent au prochain démarrage. Une nouvelle installation renseigne 0.0.0.0 afin que les autres appareils du réseau puissent se connecter. Un hôte vide reste sur cet ordinateur.",

    select_data_dir: "Sélectionner le répertoire de données RustFS",
    data_path: "Chemin des données",
    data_path_placeholder: "Sélectionnez ou déposez un dossier de données",
    browse: "Parcourir",
    host: "Hôte",
    api_port: "Port API",
    console: "Console",
    console_port: "Port de la console",
    change_admin: "Changer l'administrateur",
    close_change_admin: "Fermer le changement d'administrateur",
    admin_note: "Ces clés seront utilisées au prochain démarrage. D'ici là, RustFS conserve les clés affichées sur l'écran principal. La valeur publique par défaut est rustfsadmin.",
    access_key: "Clé d'accès",
    secret_key: "Clé secrète",
    launching: "Démarrage…",
    stop_rustfs: "Arrêter RustFS",
    launch_rustfs: "Démarrer RustFS",
    external_listener: "Un autre programme écoute déjà sur cet hôte et ce port, mais ce lanceur ne l'a pas démarré. Changez le port dans Paramètres ou arrêtez ce programme, puis démarrez.",
    api_port_invalid: "Le port API doit être un nombre compris entre 1 et 65535.",
    console_port_invalid: "Le port de la console doit être un nombre compris entre 1 et 65535.",
    port_conflict: "Le port API et le port de la console doivent être différents.",
    data_path_required: "Le chemin des données est obligatoire.",

    endpoint_region: "Adresse d'écoute et identifiants",
    reachability_lan: "Les autres appareils de votre réseau peuvent joindre ce point d'accès. Les clés ci-dessous sont celles utilisées par RustFS.",
    reachability_local: "Seul cet ordinateur peut joindre ce point d'accès. Les clés ci-dessous sont celles utilisées par RustFS.",
    listening: "Écoute",
    open_in_browser: "Ouvrir dans le navigateur",
    copy: "Copier",
    copied: "Copié",
    copy_failed: "Impossible de copier. Sélectionnez la valeur et copiez-la manuellement.",
    open_console: "Ouvrir la console",
    view_logs: "Voir les journaux",
    hide_logs: "Masquer les journaux",
    logs_region: "Journaux",

    current_version: |version| format!("Version actuelle : {version}"),
    current_version_with_rustfs: |launcher, rustfs| {
        format!("Version actuelle : {launcher} · RustFS {rustfs}")
    },
    product_links: "Liens du produit",
    upgrade: "Mise à jour",
    close_upgrade: "Fermer la mise à jour",
    website: "Site web",
    docs: "Docs",
    github: "GitHub",
    checking: "Vérification…",
    check_for_updates: "Vérifier",
    unknown: "inconnue",
    source_installed: "installée",
    source_bundled: "intégrée",
    launcher_version: |version| format!("Lanceur {version}"),
    rustfs_version: |version, source| format!("RustFS {version} ({source})"),
    no_release_notes: "Aucune note de version pour cette version.",
    installing: "Installation…",
    update_launcher: "Mettre à jour le lanceur",
    update_rustfs: "Mettre à jour RustFS",
    launcher_up_to_date: "Le lanceur est à jour.",
    rustfs_up_to_date: "RustFS est à jour.",
    verified_download: "Téléchargement vérifié depuis le flux de versions RustFS.",
    launcher_progress: "Progression de la mise à jour du lanceur",
    rustfs_progress: "Progression de la mise à jour de RustFS",

    launching_rustfs: "Démarrage de RustFS…",
    launch_succeeded: "RustFS a démarré avec succès !",
    stopping_rustfs: "Arrêt de RustFS…",
    rustfs_stopped: "RustFS est arrêté",
    all_up_to_date: "Vous utilisez les dernières versions",
    update_available: |found| format!("Mise à jour disponible : {found}"),
    confirm_launcher_update: "La mise à jour du lanceur arrête le service RustFS qu'il gère et redémarre l'application une fois l'installation terminée. Continuer ?",
    confirm_rustfs_update: "La mise à jour de RustFS arrête le service géré par ce lanceur. Redémarrez-le ensuite pour utiliser la nouvelle version. Continuer ?",
    installing_launcher_update: "Téléchargement et installation de la mise à jour du lanceur…",
    installing_rustfs_update: "Téléchargement et installation de la mise à jour de RustFS…",
    dismiss_notification: "Fermer la notification",

    reason_launch_default: "RustFS n'a pas démarré.",
    reason_stop_default: "RustFS ne s'est pas arrêté.",
    reason_upgrade_default: "La mise à jour ne s'est pas terminée.",
    reason_check_default: "La recherche de mises à jour ne s'est pas terminée.",
    reason_already_running: "RustFS est déjà en cours d'exécution.",
    reason_no_update: "Aucune mise à jour n'est disponible.",
    reason_not_connected: "La fenêtre n'est pas connectée à l'application de bureau.",
    reason_unreadable_result: "Le lanceur n'a pas pu lire le résultat de l'action.",

    fix_launch_default: "Vérifiez le chemin des données et les ports, puis relancez.",
    fix_stop_default: "Cliquez à nouveau sur « Arrêter RustFS ». Si le service tourne toujours, quittez le lanceur depuis le menu de la barre d'état.",
    fix_upgrade_default: "Relancez la mise à jour. Une mise à jour échouée ne modifie pas la version installée.",
    fix_check_default: "Vérifiez la connexion réseau, puis cliquez à nouveau sur « Vérifier ».",
    fix_choose_data_folder: "Choisissez un dossier avec « Parcourir » ou saisissez le chemin, puis relancez.",
    fix_create_folder: "Créez ce dossier, puis relancez. Le lanceur ne le crée pas pour vous.",
    fix_folder_not_file: "Choisissez un dossier, pas un fichier, puis relancez.",
    fix_pick_local_folder: "Choisissez un dossier local avec « Parcourir », puis relancez.",
    fix_writable_parent: "Choisissez un dossier de données dont le répertoire parent est accessible en écriture, puis relancez.",
    fix_port_in_use: "Choisissez un autre port ou arrêtez le programme qui l'écoute, puis relancez.",
    fix_different_ports: "Définissez des ports différents pour l'API et la console, puis relancez.",
    fix_port_number: "Saisissez un nombre entier entre 1 et 65535, puis relancez.",
    fix_bind_host: "Utilisez 127.0.0.1, localhost ou une adresse IP comme 0.0.0.0, puis relancez.",
    fix_public_credentials: "Liez le service à 127.0.0.1 ou définissez une clé d'accès et une clé secrète uniques. Des identifiants vides reviennent aussi à rustfsadmin.",
    fix_already_running: "Cliquez sur « Arrêter RustFS » si ce lanceur l'a démarré. Sinon, changez de port ou arrêtez l'autre programme, puis relancez.",
    fix_binary_not_found: "Réinstallez RustFS Launcher, ou installez RustFS via « Vérifier », puis relancez.",
    fix_binary_exec: "Réinstallez RustFS Launcher pour restaurer le binaire intégré, puis relancez.",
    fix_launch_exit: "Vérifiez que le dossier de données existe et que le port est libre, puis relancez. RustFS écrit plus de détails dans le dossier logs à côté du répertoire de données.",
    fix_retry_rustfs_update: "Relancez la mise à jour de RustFS. L'installation actuelle n'a pas été modifiée.",
    fix_network_update: "Vérifiez la connexion réseau, puis réessayez. Une mise à jour échouée ne modifie pas la version installée.",
    fix_no_update: "Ce lanceur est déjà à jour. Démarrez RustFS avec la version installée.",
    fix_asset_missing: "Réessayez plus tard. La version installée n'a pas été modifiée.",
    fix_open_from_shell: "Ouvrez RustFS Launcher depuis le menu Démarrer ou le dossier Applications, puis relancez.",

    log_sources: "Sources des journaux",
    app_logs: "Journaux",
    rustfs_output: "Sortie RustFS",
    auto_scroll: "Défilement auto",
    clear_logs: "Effacer les journaux",
    clear: "Effacer",
    no_logs: "Aucun journal disponible",
};

pub const JA: Strings = Strings {
    banner_text: "デスクトップ版は本番環境での使用を推奨しません",
    banner_close: "通知を閉じる",
    language: "言語",

    service_online: "サービス稼働中",
    service_offline: "サービス停止中",
    settings: "設定",
    close_settings: "設定を閉じる",
    settings_note: "ホスト、ポート、コンソールは次回起動時に反映されます。新規インストール時は 0.0.0.0 が設定され、同じネットワークの他の端末からも接続できます。空欄にするとこのコンピューターのみになります。",

    select_data_dir: "RustFS のデータディレクトリを選択",
    data_path: "データパス",
    data_path_placeholder: "データフォルダーを選択またはドロップ",
    browse: "参照",
    host: "ホスト",
    api_port: "API ポート",
    console: "コンソール",
    console_port: "コンソールポート",
    change_admin: "管理者を変更",
    close_change_admin: "管理者の変更を閉じる",
    admin_note: "これらのキーは次回起動時に使われます。それまで RustFS はメイン画面に表示されているキーを使い続けます。公開されている既定値は rustfsadmin です。",
    access_key: "アクセスキー",
    secret_key: "シークレットキー",
    launching: "起動中…",
    stop_rustfs: "RustFS を停止",
    launch_rustfs: "RustFS を起動",
    external_listener: "このホストとポートは別のプログラムが使用中ですが、このランチャーが起動したものではありません。「設定」でポートを変更するか、そのプログラムを停止してから起動してください。",
    api_port_invalid: "API ポートには 1〜65535 の数値を指定してください。",
    console_port_invalid: "コンソールポートには 1〜65535 の数値を指定してください。",
    port_conflict: "API ポートとコンソールポートには異なる値を指定してください。",
    data_path_required: "データパスを入力してください。",

    endpoint_region: "リッスンアドレスと認証情報",
    reachability_lan: "同じネットワーク上の他の端末からもこのエンドポイントに接続できます。下のキーは RustFS が現在使用しているものです。",
    reachability_local: "このコンピューターからのみこのエンドポイントに接続できます。下のキーは RustFS が現在使用しているものです。",
    listening: "リッスン",
    open_in_browser: "ブラウザーで開く",
    copy: "コピー",
    copied: "コピーしました",
    copy_failed: "コピーできませんでした。値を選択して手動でコピーしてください。",
    open_console: "コンソールを開く",
    view_logs: "ログを表示",
    hide_logs: "ログを隠す",
    logs_region: "ログ",

    current_version: |version| format!("現在のバージョン：{version}"),
    current_version_with_rustfs: |launcher, rustfs| {
        format!("現在のバージョン：{launcher} · RustFS {rustfs}")
    },
    product_links: "製品リンク",
    upgrade: "アップグレード",
    close_upgrade: "アップグレードを閉じる",
    website: "公式サイト",
    docs: "ドキュメント",
    github: "GitHub",
    checking: "確認中…",
    check_for_updates: "更新を確認",
    unknown: "不明",
    source_installed: "アプリ内でインストール",
    source_bundled: "同梱",
    launcher_version: |version| format!("ランチャー {version}"),
    rustfs_version: |version, source| format!("RustFS {version}（{source}）"),
    no_release_notes: "このバージョンのリリースノートはありません。",
    installing: "インストール中…",
    update_launcher: "ランチャーを更新",
    update_rustfs: "RustFS を更新",
    launcher_up_to_date: "ランチャーは最新です。",
    rustfs_up_to_date: "RustFS は最新です。",
    verified_download: "RustFS のリリースフィードから取得し、検証済みです。",
    launcher_progress: "ランチャーの更新の進行状況",
    rustfs_progress: "RustFS の更新の進行状況",

    launching_rustfs: "RustFS を起動しています…",
    launch_succeeded: "RustFS を起動しました",
    stopping_rustfs: "RustFS を停止しています…",
    rustfs_stopped: "RustFS を停止しました",
    all_up_to_date: "すべて最新バージョンです",
    update_available: |found| format!("更新があります：{found}"),
    confirm_launcher_update: "ランチャーを更新すると、管理中の RustFS サービスが停止し、インストール完了後にアプリが再起動します。続行しますか？",
    confirm_rustfs_update: "RustFS を更新すると、このランチャーが管理するサービスが停止します。新しいバージョンを使うには、更新後にもう一度起動してください。続行しますか？",
    installing_launcher_update: "ランチャーの更新をダウンロードしてインストールしています…",
    installing_rustfs_update: "RustFS の更新をダウンロードしてインストールしています…",
    dismiss_notification: "通知を閉じる",

    reason_launch_default: "RustFS を起動できませんでした。",
    reason_stop_default: "RustFS を停止できませんでした。",
    reason_upgrade_default: "更新が完了しませんでした。",
    reason_check_default: "更新の確認が完了しませんでした。",
    reason_already_running: "RustFS はすでに実行中です。",
    reason_no_update: "利用できる更新はありません。",
    reason_not_connected: "ウィンドウがデスクトップアプリに接続されていません。",
    reason_unreadable_result: "ランチャーが操作の結果を読み取れませんでした。",

    fix_launch_default: "データパスとポートを確認してから、もう一度起動してください。",
    fix_stop_default: "もう一度「RustFS を停止」を押してください。サービスが動き続ける場合は、トレイメニューからランチャーを終了してください。",
    fix_upgrade_default: "もう一度更新してください。更新に失敗しても、インストール済みのバージョンは変更されません。",
    fix_check_default: "ネットワーク接続を確認してから、もう一度「更新を確認」を押してください。",
    fix_choose_data_folder: "「参照」でフォルダーを選ぶかパスを入力してから、もう一度起動してください。",
    fix_create_folder: "このフォルダーを作成してから、もう一度起動してください。ランチャーは自動では作成しません。",
    fix_folder_not_file: "ファイルではなくフォルダーを選んでから、もう一度起動してください。",
    fix_pick_local_folder: "「参照」でローカルのフォルダーを選んでから、もう一度起動してください。",
    fix_writable_parent: "親ディレクトリに書き込みできるデータフォルダーを選んでから、もう一度起動してください。",
    fix_port_in_use: "別のポートを選ぶか、そのポートを使っているプログラムを停止してから、もう一度起動してください。",
    fix_different_ports: "API ポートとコンソールポートに異なる値を設定してから、もう一度起動してください。",
    fix_port_number: "1〜65535 の整数を入力してから、もう一度起動してください。",
    fix_bind_host: "127.0.0.1、localhost、または 0.0.0.0 のような IP アドレスを指定してから、もう一度起動してください。",
    fix_public_credentials: "127.0.0.1 にバインドするか、固有のアクセスキーとシークレットキーを設定してください。空欄の場合も rustfsadmin が使われます。",
    fix_already_running: "このランチャーで起動した場合は「RustFS を停止」を押してください。そうでない場合は、ポートを変更するか別のプログラムを停止してから、もう一度起動してください。",
    fix_binary_not_found: "RustFS Launcher を再インストールするか、「更新を確認」から RustFS をインストールしてから、もう一度起動してください。",
    fix_binary_exec: "RustFS Launcher を再インストールして同梱のバイナリを復元してから、もう一度起動してください。",
    fix_launch_exit: "データフォルダーが存在し、ポートが空いていることを確認してから、もう一度起動してください。詳しい情報はデータディレクトリの隣にある logs フォルダーに出力されます。",
    fix_retry_rustfs_update: "もう一度 RustFS を更新してください。現在のインストールは変更されていません。",
    fix_network_update: "ネットワーク接続を確認してから、もう一度お試しください。更新に失敗しても、インストール済みのバージョンは変更されません。",
    fix_no_update: "ランチャーはすでに最新です。インストール済みのバージョンで RustFS を起動してください。",
    fix_asset_missing: "しばらくしてからもう一度お試しください。インストール済みのバージョンは変更されていません。",
    fix_open_from_shell: "スタートメニューまたは「アプリケーション」フォルダーから RustFS Launcher を開いてから、もう一度起動してください。",

    log_sources: "ログの種類",
    app_logs: "アプリログ",
    rustfs_output: "RustFS 出力",
    auto_scroll: "自動スクロール",
    clear_logs: "ログを消去",
    clear: "消去",
    no_logs: "ログはありません",
};

#[cfg(test)]
mod tests {
    use super::{resolve_lang, ActionLabel, Lang};

    #[test]
    fn saved_choice_wins_over_the_system_language() {
        assert_eq!(resolve_lang(Some("en"), Some("zh-CN")), Lang::En);
        assert_eq!(resolve_lang(Some("zh-CN"), Some("en-US")), Lang::Zh);
    }

    #[test]
    fn system_language_matches_on_the_primary_subtag() {
        assert_eq!(resolve_lang(None, Some("zh-CN")), Lang::Zh);
        assert_eq!(resolve_lang(None, Some("zh-Hant-TW")), Lang::Zh);
        assert_eq!(resolve_lang(None, Some("ZH")), Lang::Zh);
        assert_eq!(resolve_lang(None, Some("en-GB")), Lang::En);
        assert_eq!(resolve_lang(None, Some("fr-CA")), Lang::Fr);
        assert_eq!(resolve_lang(None, Some("ja_JP")), Lang::Ja);
        assert_eq!(resolve_lang(None, Some("ja")), Lang::Ja);
    }

    #[test]
    fn unknown_or_missing_languages_fall_back_to_english() {
        assert_eq!(resolve_lang(None, Some("de-DE")), Lang::En);
        assert_eq!(resolve_lang(Some("garbage"), Some("ko")), Lang::En);
        assert_eq!(resolve_lang(None, Some("frisian")), Lang::En);
        assert_eq!(resolve_lang(Some("garbage"), Some("zh-CN")), Lang::Zh);
        assert_eq!(resolve_lang(None, None), Lang::En);
    }

    #[test]
    fn codes_round_trip_through_storage() {
        for lang in Lang::ALL {
            assert_eq!(resolve_lang(Some(lang.code()), None), lang);
        }
    }

    #[test]
    fn formatted_messages_keep_their_arguments() {
        for strings in Lang::ALL.map(Lang::strings) {
            assert!((strings.current_version)("1.0.6").contains("1.0.6"));
            let combined = (strings.current_version_with_rustfs)("1.0.6", "1.0.1");
            assert!(combined.contains("1.0.6"));
            assert!(combined.contains("1.0.1"));
            assert!((strings.rustfs_version)("1.0.1", "x").contains("1.0.1"));
            assert!((strings.update_available)("1.0.7").contains("1.0.7"));
        }
    }

    #[test]
    fn action_labels_follow_the_language() {
        assert_eq!(
            Lang::En.strings().action_label(ActionLabel::Stop),
            "Stop RustFS"
        );
        assert_eq!(
            Lang::Zh.strings().action_label(ActionLabel::Launch),
            "启动 RustFS"
        );
        assert_eq!(
            Lang::Zh.strings().action_label(ActionLabel::Launching),
            "启动中…"
        );
        assert_eq!(
            Lang::Fr.strings().action_label(ActionLabel::Stop),
            "Arrêter RustFS"
        );
        assert_eq!(
            Lang::Ja.strings().action_label(ActionLabel::Launch),
            "RustFS を起動"
        );
    }
}
