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

    // Hero and summary cards
    pub control_plane: &'static str,
    pub service_online: &'static str,
    pub service_offline: &'static str,
    pub managed_by_launcher: &'static str,
    pub detected_externally: &'static str,
    pub ready_to_launch: &'static str,
    pub open_api_title: &'static str,
    pub open_console_title: &'static str,
    pub api_label: &'static str,
    pub console_label: &'static str,
    pub disabled: &'static str,
    pub mode_label: &'static str,
    pub mode_locked: &'static str,
    pub mode_editable: &'static str,

    // Updates card
    pub software_update: &'static str,
    pub version_and_updates: &'static str,
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
    pub rustfs_exited: fn(&str) -> String,
    pub launching_rustfs: &'static str,
    pub not_in_tauri: &'static str,
    pub launch_succeeded: &'static str,
    pub launch_failed: fn(&str) -> String,
    pub launch_command_failed: &'static str,
    pub stopping_rustfs: &'static str,
    pub rustfs_stopped: &'static str,
    pub stop_failed: fn(&str) -> String,
    pub stop_response_unreadable: &'static str,
    pub launcher_check_failed_see_logs: &'static str,
    pub launcher_check_failed: fn(&str) -> String,
    pub rustfs_check_failed_see_logs: &'static str,
    pub rustfs_check_failed: fn(&str) -> String,
    pub all_up_to_date: &'static str,
    pub update_available: fn(&str) -> String,
    pub confirm_launcher_update: &'static str,
    pub confirm_rustfs_update: &'static str,
    pub installing_launcher_update: &'static str,
    pub launcher_update_failed_see_logs: &'static str,
    pub launcher_update_failed: fn(&str) -> String,
    pub installing_rustfs_update: &'static str,
    pub rustfs_update_failed_see_logs: &'static str,
    pub rustfs_update_failed: fn(&str) -> String,
    pub dismiss_notification: &'static str,

    // Configuration form
    pub select_data_dir: &'static str,
    pub config_locked: &'static str,
    pub storage: &'static str,
    pub dataset_mount: &'static str,
    pub data_path: &'static str,
    pub data_path_placeholder: &'static str,
    pub browse: &'static str,
    pub network: &'static str,
    pub ports_and_access: &'static str,
    pub api_port: &'static str,
    pub host: &'static str,
    pub console_endpoint: &'static str,
    pub console_port: &'static str,
    pub console_bind: &'static str,
    pub security: &'static str,
    pub credentials: &'static str,
    pub access_key: &'static str,
    pub access_key_placeholder: &'static str,
    pub secret_key: &'static str,
    pub secret_key_placeholder: &'static str,
    pub show_secret_key: &'static str,
    pub hide_secret_key: &'static str,
    pub launching: &'static str,
    pub stop_rustfs: &'static str,
    pub launch_rustfs: &'static str,
    pub external_listener: &'static str,
    pub api_port_invalid: &'static str,
    pub console_port_invalid: &'static str,
    pub port_conflict: &'static str,
    pub data_path_required: &'static str,
    pub public_default_credentials: fn(&str) -> String,

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

    control_plane: "Control Plane",
    service_online: "Service Online",
    service_offline: "Service Offline",
    managed_by_launcher: "Managed by Launcher",
    detected_externally: "Detected Externally",
    ready_to_launch: "Ready to Launch",
    open_api_title: "Open API endpoint",
    open_console_title: "Open Console",
    api_label: "API",
    console_label: "Console",
    disabled: "Disabled",
    mode_label: "Mode",
    mode_locked: "Locked",
    mode_editable: "Editable",

    software_update: "Software Update",
    version_and_updates: "Version & Updates",
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

    rustfs_exited: |code| format!("RustFS exited with code: {code}"),
    launching_rustfs: "Launching RustFS...",
    not_in_tauri: "Error: Not running in Tauri environment",
    launch_succeeded: "RustFS launched successfully!",
    launch_failed: |message| format!("Launch failed: {message}"),
    launch_command_failed: "RustFS launch command failed",
    stopping_rustfs: "Stopping RustFS...",
    rustfs_stopped: "RustFS stopped",
    stop_failed: |message| format!("Failed to stop: {message}"),
    stop_response_unreadable: "Failed to parse stop response",
    launcher_check_failed_see_logs: "Launcher update check failed. See App Logs for details.",
    launcher_check_failed: |message| format!("Launcher update check failed: {message}"),
    rustfs_check_failed_see_logs: "RustFS update check failed. See App Logs for details.",
    rustfs_check_failed: |message| format!("RustFS update check failed: {message}"),
    all_up_to_date: "You are running the latest versions",
    update_available: |found| format!("Update available: {found}"),
    confirm_launcher_update: "Updating the launcher stops the RustFS service it manages and restarts the app once the update is installed. Continue?",
    confirm_rustfs_update: "Updating RustFS stops the service managed by this launcher. Launch it again afterwards to use the new build. Continue?",
    installing_launcher_update: "Downloading and installing the launcher update…",
    launcher_update_failed_see_logs: "Launcher update failed. See App Logs for details.",
    launcher_update_failed: |message| format!("Launcher update failed: {message}"),
    installing_rustfs_update: "Downloading and installing the RustFS update…",
    rustfs_update_failed_see_logs: "RustFS update failed. See App Logs for details.",
    rustfs_update_failed: |message| format!("RustFS update failed: {message}"),
    dismiss_notification: "Dismiss notification",

    select_data_dir: "Select RustFS Data Directory",
    config_locked: "Configuration Locked",
    storage: "Storage",
    dataset_mount: "Dataset Mount",
    data_path: "Data Path",
    data_path_placeholder: "Select or drop data directory...",
    browse: "Browse",
    network: "Network",
    ports_and_access: "Ports & Access",
    api_port: "API Port",
    host: "Host",
    console_endpoint: "Console Endpoint",
    console_port: "Console Port",
    console_bind: "Console Bind",
    security: "Security",
    credentials: "Credentials",
    access_key: "Access Key",
    access_key_placeholder: "Access key",
    secret_key: "Secret Key",
    secret_key_placeholder: "Secret key",
    show_secret_key: "Show secret key",
    hide_secret_key: "Hide secret key",
    launching: "Launching…",
    stop_rustfs: "Stop RustFS",
    launch_rustfs: "Launch RustFS",
    external_listener: "Something is already listening on this host and port, but this launcher did not start it. Change the port, or stop that program, then launch.",
    api_port_invalid: "API port must be a number between 1 and 65535",
    console_port_invalid: "Console port must be a number between 1 and 65535",
    port_conflict: "API port and console port must be different",
    data_path_required: "Data path is required",
    public_default_credentials: |host| format!(
        "Refusing to expose the default rustfsadmin credentials on {host}. Bind to 127.0.0.1, or set a unique access key and secret key. Empty credentials also fall back to rustfsadmin"
    ),

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

    control_plane: "控制面板",
    service_online: "服务在线",
    service_offline: "服务离线",
    managed_by_launcher: "由启动器管理",
    detected_externally: "检测到外部实例",
    ready_to_launch: "可以启动",
    open_api_title: "打开 API 地址",
    open_console_title: "打开控制台",
    api_label: "API",
    console_label: "控制台",
    disabled: "未启用",
    mode_label: "模式",
    mode_locked: "已锁定",
    mode_editable: "可编辑",

    software_update: "软件更新",
    version_and_updates: "版本与更新",
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

    rustfs_exited: |code| format!("RustFS 已退出，退出码：{code}"),
    launching_rustfs: "正在启动 RustFS…",
    not_in_tauri: "错误：当前不在 Tauri 环境中运行",
    launch_succeeded: "RustFS 启动成功！",
    launch_failed: |message| format!("启动失败：{message}"),
    launch_command_failed: "RustFS 启动命令执行失败",
    stopping_rustfs: "正在停止 RustFS…",
    rustfs_stopped: "RustFS 已停止",
    stop_failed: |message| format!("停止失败：{message}"),
    stop_response_unreadable: "无法解析停止命令的返回结果",
    launcher_check_failed_see_logs: "启动器更新检查失败，详情请查看应用日志。",
    launcher_check_failed: |message| format!("启动器更新检查失败：{message}"),
    rustfs_check_failed_see_logs: "RustFS 更新检查失败，详情请查看应用日志。",
    rustfs_check_failed: |message| format!("RustFS 更新检查失败：{message}"),
    all_up_to_date: "已是最新版本",
    update_available: |found| format!("有可用更新：{found}"),
    confirm_launcher_update:
        "更新启动器会停止它所管理的 RustFS 服务，并在安装完成后重启应用。是否继续？",
    confirm_rustfs_update:
        "更新 RustFS 会停止由本启动器管理的服务，更新后需要重新启动才能使用新版本。是否继续？",
    installing_launcher_update: "正在下载并安装启动器更新…",
    launcher_update_failed_see_logs: "启动器更新失败，详情请查看应用日志。",
    launcher_update_failed: |message| format!("启动器更新失败：{message}"),
    installing_rustfs_update: "正在下载并安装 RustFS 更新…",
    rustfs_update_failed_see_logs: "RustFS 更新失败，详情请查看应用日志。",
    rustfs_update_failed: |message| format!("RustFS 更新失败：{message}"),
    dismiss_notification: "关闭通知",

    select_data_dir: "选择 RustFS 数据目录",
    config_locked: "配置已锁定",
    storage: "存储",
    dataset_mount: "数据目录",
    data_path: "数据路径",
    data_path_placeholder: "选择或拖入数据目录…",
    browse: "浏览",
    network: "网络",
    ports_and_access: "端口与访问",
    api_port: "API 端口",
    host: "监听地址",
    console_endpoint: "控制台",
    console_port: "控制台端口",
    console_bind: "控制台地址",
    security: "安全",
    credentials: "访问凭证",
    access_key: "Access Key",
    access_key_placeholder: "Access Key",
    secret_key: "Secret Key",
    secret_key_placeholder: "Secret Key",
    show_secret_key: "显示 Secret Key",
    hide_secret_key: "隐藏 Secret Key",
    launching: "启动中…",
    stop_rustfs: "停止 RustFS",
    launch_rustfs: "启动 RustFS",
    external_listener:
        "该地址和端口已被其他程序监听，但不是由本启动器启动的。请更换端口或停止该程序后再启动。",
    api_port_invalid: "API 端口必须是 1 到 65535 之间的数字",
    console_port_invalid: "控制台端口必须是 1 到 65535 之间的数字",
    port_conflict: "API 端口和控制台端口不能相同",
    data_path_required: "请填写数据路径",
    public_default_credentials: |host| {
        format!(
        "不允许在 {host} 上使用默认的 rustfsadmin 凭证对外提供服务。请绑定到 127.0.0.1，或设置独立的 Access Key 和 Secret Key。留空同样会回退为 rustfsadmin"
    )
    },

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

    control_plane: "Panneau de contrôle",
    service_online: "Service en ligne",
    service_offline: "Service hors ligne",
    managed_by_launcher: "Géré par le lanceur",
    detected_externally: "Détecté en externe",
    ready_to_launch: "Prêt à démarrer",
    open_api_title: "Ouvrir le point d'accès API",
    open_console_title: "Ouvrir la console",
    api_label: "API",
    console_label: "Console",
    disabled: "Désactivée",
    mode_label: "Mode",
    mode_locked: "Verrouillé",
    mode_editable: "Modifiable",

    software_update: "Mise à jour",
    version_and_updates: "Versions et mises à jour",
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

    rustfs_exited: |code| format!("RustFS s'est arrêté avec le code : {code}"),
    launching_rustfs: "Démarrage de RustFS…",
    not_in_tauri: "Erreur : l'application ne s'exécute pas dans Tauri",
    launch_succeeded: "RustFS a démarré avec succès !",
    launch_failed: |message| format!("Échec du démarrage : {message}"),
    launch_command_failed: "La commande de démarrage de RustFS a échoué",
    stopping_rustfs: "Arrêt de RustFS…",
    rustfs_stopped: "RustFS est arrêté",
    stop_failed: |message| format!("Échec de l'arrêt : {message}"),
    stop_response_unreadable: "Impossible de lire la réponse à la commande d'arrêt",
    launcher_check_failed_see_logs: "La recherche de mises à jour du lanceur a échoué. Consultez l'onglet Journaux pour plus de détails.",
    launcher_check_failed: |message| format!("La recherche de mises à jour du lanceur a échoué : {message}"),
    rustfs_check_failed_see_logs: "La recherche de mises à jour de RustFS a échoué. Consultez l'onglet Journaux pour plus de détails.",
    rustfs_check_failed: |message| format!("La recherche de mises à jour de RustFS a échoué : {message}"),
    all_up_to_date: "Vous utilisez les dernières versions",
    update_available: |found| format!("Mise à jour disponible : {found}"),
    confirm_launcher_update: "La mise à jour du lanceur arrête le service RustFS qu'il gère et redémarre l'application une fois l'installation terminée. Continuer ?",
    confirm_rustfs_update: "La mise à jour de RustFS arrête le service géré par ce lanceur. Redémarrez-le ensuite pour utiliser la nouvelle version. Continuer ?",
    installing_launcher_update: "Téléchargement et installation de la mise à jour du lanceur…",
    launcher_update_failed_see_logs: "La mise à jour du lanceur a échoué. Consultez l'onglet Journaux pour plus de détails.",
    launcher_update_failed: |message| format!("La mise à jour du lanceur a échoué : {message}"),
    installing_rustfs_update: "Téléchargement et installation de la mise à jour de RustFS…",
    rustfs_update_failed_see_logs: "La mise à jour de RustFS a échoué. Consultez l'onglet Journaux pour plus de détails.",
    rustfs_update_failed: |message| format!("La mise à jour de RustFS a échoué : {message}"),
    dismiss_notification: "Fermer la notification",

    select_data_dir: "Sélectionner le répertoire de données RustFS",
    config_locked: "Configuration verrouillée",
    storage: "Stockage",
    dataset_mount: "Répertoire de données",
    data_path: "Chemin des données",
    data_path_placeholder: "Sélectionnez ou déposez un répertoire…",
    browse: "Parcourir",
    network: "Réseau",
    ports_and_access: "Ports et accès",
    api_port: "Port API",
    host: "Hôte",
    console_endpoint: "Console",
    console_port: "Port de la console",
    console_bind: "Adresse de la console",
    security: "Sécurité",
    credentials: "Identifiants",
    access_key: "Clé d'accès",
    access_key_placeholder: "Clé d'accès",
    secret_key: "Clé secrète",
    secret_key_placeholder: "Clé secrète",
    show_secret_key: "Afficher la clé secrète",
    hide_secret_key: "Masquer la clé secrète",
    launching: "Démarrage…",
    stop_rustfs: "Arrêter RustFS",
    launch_rustfs: "Démarrer RustFS",
    external_listener: "Un autre programme écoute déjà sur cet hôte et ce port, mais ce lanceur ne l'a pas démarré. Changez de port ou arrêtez ce programme, puis démarrez.",
    api_port_invalid: "Le port API doit être un nombre compris entre 1 et 65535",
    console_port_invalid: "Le port de la console doit être un nombre compris entre 1 et 65535",
    port_conflict: "Le port API et le port de la console doivent être différents",
    data_path_required: "Le chemin des données est obligatoire",
    public_default_credentials: |host| format!(
        "Refus d'exposer les identifiants par défaut rustfsadmin sur {host}. Liez le service à 127.0.0.1 ou définissez une clé d'accès et une clé secrète uniques. Des identifiants vides reviennent aussi à rustfsadmin"
    ),

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

    control_plane: "コントロールパネル",
    service_online: "サービス稼働中",
    service_offline: "サービス停止中",
    managed_by_launcher: "ランチャーで管理中",
    detected_externally: "外部のインスタンスを検出",
    ready_to_launch: "起動可能",
    open_api_title: "API エンドポイントを開く",
    open_console_title: "コンソールを開く",
    api_label: "API",
    console_label: "コンソール",
    disabled: "無効",
    mode_label: "モード",
    mode_locked: "ロック中",
    mode_editable: "編集可能",

    software_update: "ソフトウェア更新",
    version_and_updates: "バージョンと更新",
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

    rustfs_exited: |code| format!("RustFS が終了しました（終了コード：{code}）"),
    launching_rustfs: "RustFS を起動しています…",
    not_in_tauri: "エラー：Tauri 環境で実行されていません",
    launch_succeeded: "RustFS を起動しました",
    launch_failed: |message| format!("起動に失敗しました：{message}"),
    launch_command_failed: "RustFS の起動コマンドが失敗しました",
    stopping_rustfs: "RustFS を停止しています…",
    rustfs_stopped: "RustFS を停止しました",
    stop_failed: |message| format!("停止に失敗しました：{message}"),
    stop_response_unreadable: "停止コマンドの応答を解析できませんでした",
    launcher_check_failed_see_logs: "ランチャーの更新確認に失敗しました。詳細はアプリログを確認してください。",
    launcher_check_failed: |message| format!("ランチャーの更新確認に失敗しました：{message}"),
    rustfs_check_failed_see_logs: "RustFS の更新確認に失敗しました。詳細はアプリログを確認してください。",
    rustfs_check_failed: |message| format!("RustFS の更新確認に失敗しました：{message}"),
    all_up_to_date: "すべて最新バージョンです",
    update_available: |found| format!("更新があります：{found}"),
    confirm_launcher_update: "ランチャーを更新すると、管理中の RustFS サービスが停止し、インストール完了後にアプリが再起動します。続行しますか？",
    confirm_rustfs_update: "RustFS を更新すると、このランチャーが管理するサービスが停止します。新しいバージョンを使うには、更新後にもう一度起動してください。続行しますか？",
    installing_launcher_update: "ランチャーの更新をダウンロードしてインストールしています…",
    launcher_update_failed_see_logs: "ランチャーの更新に失敗しました。詳細はアプリログを確認してください。",
    launcher_update_failed: |message| format!("ランチャーの更新に失敗しました：{message}"),
    installing_rustfs_update: "RustFS の更新をダウンロードしてインストールしています…",
    rustfs_update_failed_see_logs: "RustFS の更新に失敗しました。詳細はアプリログを確認してください。",
    rustfs_update_failed: |message| format!("RustFS の更新に失敗しました：{message}"),
    dismiss_notification: "通知を閉じる",

    select_data_dir: "RustFS のデータディレクトリを選択",
    config_locked: "設定はロックされています",
    storage: "ストレージ",
    dataset_mount: "データディレクトリ",
    data_path: "データパス",
    data_path_placeholder: "データディレクトリを選択またはドロップ…",
    browse: "参照",
    network: "ネットワーク",
    ports_and_access: "ポートとアクセス",
    api_port: "API ポート",
    host: "ホスト",
    console_endpoint: "コンソール",
    console_port: "コンソールポート",
    console_bind: "コンソールのアドレス",
    security: "セキュリティ",
    credentials: "認証情報",
    access_key: "アクセスキー",
    access_key_placeholder: "アクセスキー",
    secret_key: "シークレットキー",
    secret_key_placeholder: "シークレットキー",
    show_secret_key: "シークレットキーを表示",
    hide_secret_key: "シークレットキーを隠す",
    launching: "起動中…",
    stop_rustfs: "RustFS を停止",
    launch_rustfs: "RustFS を起動",
    external_listener: "このホストとポートは別のプログラムが使用中ですが、このランチャーが起動したものではありません。ポートを変更するか、そのプログラムを停止してから起動してください。",
    api_port_invalid: "API ポートには 1〜65535 の数値を指定してください",
    console_port_invalid: "コンソールポートには 1〜65535 の数値を指定してください",
    port_conflict: "API ポートとコンソールポートには異なる値を指定してください",
    data_path_required: "データパスを入力してください",
    public_default_credentials: |host| format!(
        "既定の rustfsadmin 認証情報を {host} で公開することはできません。127.0.0.1 にバインドするか、固有のアクセスキーとシークレットキーを設定してください。空欄の場合も rustfsadmin が使われます"
    ),

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
            let message = (strings.public_default_credentials)("0.0.0.0");
            assert!(message.contains("0.0.0.0"));
            assert!(message.contains("rustfsadmin"));
            assert!((strings.rustfs_version)("1.0.1", "x").contains("1.0.1"));
            assert!((strings.launch_failed)("port busy").contains("port busy"));
        }
        assert!((Lang::En.strings().public_default_credentials)("0.0.0.0")
            .contains("unique access key"));
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
