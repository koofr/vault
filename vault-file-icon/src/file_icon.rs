use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileIconTheme {
    pub color_graphics_a: String,
    pub color_graphics_b: String,
    pub color_graphics_c: String,
    pub color_graphics_d: String,
    pub color_graphics_e: String,
    pub color_graphics_f: String,
    pub color_graphics_g: String,
    pub color_graphics_h: String,
    pub color_graphics_i: String,
    pub color_graphics_j: String,
    pub color_graphics_k: String,
    pub color_graphics_l: String,
    pub color_graphics_m: String,

    pub color_white: String,
    pub color_black: String,
    pub color_android: String,
    pub color_android_split: String,
    pub color_ios: String,
    pub color_ios_split: String,
    pub color_link: String,
    pub color_transfer: String,
    pub color_export: String,
    pub color_import: String,
}

impl Default for FileIconTheme {
    fn default() -> Self {
        Self {
            color_graphics_a: "#0692c2".into(),
            color_graphics_b: "#e36d1e".into(),
            color_graphics_c: "#df533e".into(),
            color_graphics_d: "#71ba05".into(),
            color_graphics_e: "#71ba05".into(),
            color_graphics_f: "#9bcb48".into(),
            color_graphics_g: "#e99308".into(),
            color_graphics_h: "#71ba05".into(),
            color_graphics_i: "#263238".into(),
            color_graphics_j: "#7e8890".into(),
            color_graphics_k: "#00aac3".into(),
            color_graphics_l: "#9160a0".into(),
            color_graphics_m: "#ff8181".into(),

            color_white: "#fff".into(),
            color_black: "#000".into(),
            color_android: "#85c808".into(),
            color_android_split: "#85c808".into(),
            color_ios: "#000000".into(),
            color_ios_split: "#c0c0c0".into(),
            color_link: "#00aac3".into(),
            color_transfer: "#1e395a".into(),
            color_export: "#00aac3".into(),
            color_import: "#9160a0".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileIconCategory {
    Generic,
    Folder,
    Archive,
    Audio,
    Code,
    Document,
    Image,
    Pdf,
    Presentation,
    Sheet,
    Text,
    Video,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileIconSize {
    Sm,
    Lg,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileIconAttrs {
    pub category: FileIconCategory,
    pub is_dl: bool,
    pub is_ul: bool,
    pub is_download_transfer: bool,
    pub is_upload_transfer: bool,
    pub is_export: bool,
    pub is_import: bool,
    pub is_android: bool,
    pub is_ios: bool,
    pub is_vault_repo: bool,
    pub is_error: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileIconProps {
    pub size: FileIconSize,
    pub attrs: FileIconAttrs,
}

struct FileIconDefFile {
    bg: String,
    border: String,
    shared_border: String,
    export_bg: String,
    import_bg: String,
    shared: String,
    link_bg: String,
    transfer_bg: String,
    error_overlay: Option<String>,
}

struct FileIconDefFolderSplit {
    normal: String,
    export: String,
    import: String,
    android: String,
    ios: String,
}

struct FileIconDefFolder {
    bg: String,
    border: String,
    split: FileIconDefFolderSplit,
    export: String,
    import: String,
    android: String,
    ios: String,
    vault_repo: String,
    link_bg: String,
    transfer_bg: String,
    error_overlay: Option<String>,
}

struct FileIconDefSize {
    width: u32,
    height: u32,
    file: FileIconDefFile,
    folder: FileIconDefFolder,
    fold: String,
    shadow: String,
    dl: String,
    ul: String,
    dl_ul: String,
    categories: HashMap<FileIconCategory, String>,
    error: String,
}

struct FileIconDefs {
    sm: FileIconDefSize,
    lg: FileIconDefSize,
}

impl FileIconDefs {
    fn generate(theme: &FileIconTheme) -> FileIconDefs {
        let FileIconTheme {
            color_graphics_a,
            color_graphics_b,
            color_graphics_c,
            color_graphics_d,
            color_graphics_e,
            color_graphics_f,
            color_graphics_g,
            color_graphics_h,
            color_graphics_i,
            color_graphics_j,
            color_graphics_m,

            color_white,
            color_black,
            color_android,
            color_android_split,
            color_ios,
            color_ios_split,
            color_link,
            color_transfer,
            color_export,
            color_import,
            ..
        } = &theme;

        let def_sm_folder_split = |color: &str| {
            format!(
                r#"<path d="M1 19h3.252c1.166 0 2.599-.548 3.339-1.455l.794-.973c.232-.284.844-.572 1.218-.572h15.438v-2H9.603c-.972 0-2.154.556-2.767 1.308l-.795.972c-.36.442-1.225.72-1.789.72H1v2z" fill="{color}" />"#
            )
        };

        let def_lg_folder_split = |color: &str| {
            format!(
                r#"<path d="M7.878 43c2.557 0 5.762-1.564 7.387-3.546l2.068-2.523c.689-.84 2.303-1.936 3.392-1.931H59v-3H20.737c-1.991-.008-4.462 1.49-5.724 3.03l-2.068 2.522C11.89 38.838 9.537 40 7.878 40H1l.003 3h6.875z" fill="{color}" />"#
            )
        };

        FileIconDefs {
            sm: FileIconDefSize {
                width: 26,
                height: 29,
                file: FileIconDefFile {
                    bg: format!(
                        r#"<path d="M16.622.673C16.221.301 15.454 0 14.901 0H.994C.445 0 0 .456 0 .998v27.004c0 .551.451.998 1.003.998h23.994C25.55 29 26 28.553 26 28V10.361c0-.553-.325-1.302-.727-1.674L16.622.673z" fill="{color_white}" />"#
                    ),
                    border: format!(
                        r#"<path d="M15.998 1H.994C1 1 1 28.002 1 28.002 1 27.999 24.997 28 24.997 28c.003 0 0-18.002 0-18.002 0-.276-.21-.759-.407-.94l-7.55-7.651C16.821 1.206 16.297 1 15.997 1zm1.755-.327l7.52 7.66c.402.372.727 1.121.727 1.674V28c0 .554-.45 1.001-1.003 1.001H1.003A1.002 1.002 0 0 1 0 28.002V.998C0 .456.445 0 .994 0h15.038c.552 0 1.32.301 1.72.673z" fill="{color_graphics_i}" />"#
                    ),
                    shared_border: format!(r#"<path d="M1 28h12V16H1z" fill="{color_white}" />"#),
                    export_bg: format!(r#"<path d="M1 17h11v11H1z" fill="{color_export}" />"#),
                    import_bg: format!(r#"<path d="M1 17h11v11H1z" fill="{color_import}" />"#),
                    shared: format!(
                        r#"<path d="M3 24.647a1.36 1.36 0 0 1 1.36-1.352h3.696c.751 0 1.36.6 1.36 1.352V26H3v-1.353zm3.15-2.305A1.66 1.66 0 0 1 4.5 20.67a1.66 1.66 0 0 1 1.65-1.671c.911 0 1.65.748 1.65 1.67a1.66 1.66 0 0 1-1.65 1.672z" fill="{color_white}" />"#
                    ),
                    link_bg: format!(r#"<path d="M16 10h9v18h-9V10z" fill="{color_link}" />"#),
                    transfer_bg: format!(
                        r#"<path d="M16 10h9v18h-9V10z" fill="{color_transfer}" />"#
                    ),
                    error_overlay: None,
                },
                folder: FileIconDefFolder {
                    bg: format!(
                        r#"<path d="M5 1h10.998c.3 0 .824.205 1.041.406l7.55 7.65c.198.183.407.667.407.942v8.357c.008 2.875.004 5.768.004 8.649a.997.997 0 0 1-.996.996H1.996A.993.993 0 0 1 1 27.004V11c0-.55.45-1 1-1 1 .01 2.001 0 3.001-.023C5.001 9.977 5.002 1 5 1z" fill="{color_white}" />"#
                    ),
                    border: format!(
                        r#"<path d="M5 0c-.55 0-1 .455-1 .998V9H2c-1.102 0-2 .898-2 2v16.004C0 28.11.893 29 1.996 29h22.008A1.997 1.997 0 0 0 26 27.004V10.008c0-.553-.325-1.304-.727-1.676L17.752.674C17.351.302 16.584 0 16.032 0zm0 1h10.998c.3 0 .824.205 1.041.406l7.55 7.65c.198.183.407.667.407.942v8.357H25v8.649a.997.997 0 0 1-.996.996H1.996A.993.993 0 0 1 1 27.004V11c0-.55.45-1.001 1-1h2v8.355h1C5.001 10.385 5.004 1 5 1z" fill="{color_graphics_i}" />"#
                    ),
                    split: FileIconDefFolderSplit {
                        normal: format!(
                            r#"<path d="M1 19h3.214c1.015 0 2.303-.67 2.948-1.46l.795-.973c.33-.405 1.069-.567 1.595-.567H25v-1H9.552c-.827 0-1.85.297-2.37.934l-.794.973C5.932 17.465 4.928 18 4.214 18H1v1z" fill="{color_graphics_i}" />"#
                        ),
                        export: def_sm_folder_split(color_export),
                        import: def_sm_folder_split(color_import),
                        android: def_sm_folder_split(color_android_split),
                        ios: def_sm_folder_split(color_ios_split),
                    },
                    export: format!(
                        r#"<path d="M10.65 19A1.66 1.66 0 0 0 9 20.67c0 .923.74 1.672 1.65 1.672.912 0 1.65-.75 1.65-1.672A1.66 1.66 0 0 0 10.65 19zm7.6 0a1.66 1.66 0 0 0-1.65 1.67c0 .923.739 1.672 1.65 1.672.911 0 1.65-.75 1.65-1.672A1.66 1.66 0 0 0 18.25 19zm-9.389 4.295A1.36 1.36 0 0 0 7.5 24.646V26h6.416v-1.354c0-.752-.608-1.351-1.36-1.351H8.862zm7.582 0c-.745 0-1.359.604-1.359 1.351V26H21.5v-1.354c0-.752-.61-1.351-1.361-1.351h-3.696z" fill="{color_export}" />"#
                    ),
                    import: format!(
                        r#"<path d="M10.65 19A1.66 1.66 0 0 0 9 20.67c0 .923.74 1.672 1.65 1.672.912 0 1.65-.75 1.65-1.672A1.66 1.66 0 0 0 10.65 19zm7.6 0a1.66 1.66 0 0 0-1.65 1.67c0 .923.739 1.672 1.65 1.672.911 0 1.65-.75 1.65-1.672A1.66 1.66 0 0 0 18.25 19zm-9.389 4.295A1.36 1.36 0 0 0 7.5 24.646V26h6.416v-1.354c0-.752-.608-1.351-1.36-1.351H8.862zm7.582 0c-.745 0-1.359.604-1.359 1.351V26H21.5v-1.354c0-.752-.61-1.351-1.361-1.351h-3.696z" fill="{color_import}" />"#
                    ),
                    android: format!(
                        r#"<path d="M17.066 18.295c-.008-.012-.034-.008-.056.008-.023.016-.034.037-.026.049l.4.56c-1.13.501-1.18 1.701-1.18 1.701h4.605s-.065-1.193-1.194-1.697l.403-.564c.008-.012-.003-.033-.026-.05-.022-.015-.048-.019-.056-.007l-.418.584a2.703 2.703 0 0 0-1.012-.193c-.42-.002-.744.08-1.026.189l-.414-.58zm.422 1.06a.277.277 0 1 1 0 .553.28.28 0 0 1-.281-.277c0-.153.126-.276.281-.276zm2.026 0c.154 0 .28.123.28.276a.28.28 0 0 1-.56 0c0-.153.125-.276.28-.276zm-4.076 1.375a.489.489 0 0 0-.493.487v2.185c0 .269.22.487.492.487a.49.49 0 0 0 .495-.487v-2.185a.49.49 0 0 0-.495-.487zm6.117 0a.489.489 0 0 0-.492.487v2.185c0 .269.22.487.492.487a.49.49 0 0 0 .494-.487v-2.185a.49.49 0 0 0-.494-.487zm-5.37.155v3.853c0 .151.123.274.276.274h.592v1.215a.49.49 0 0 0 .494.486.489.489 0 0 0 .492-.486v-1.215h.914v1.215a.49.49 0 0 0 .494.486.489.489 0 0 0 .492-.486v-1.215h.592a.276.276 0 0 0 .278-.274v-1.97h.007v-1.883h-4.63z" fill="{color_android}" fillRule="evenodd" />"#
                    ),
                    ios: format!(
                        r#"<path d="M20.111 18.293c-.041-.035-.808.106-1.283.637-.475.53-.444 1.338-.402 1.373.042.035.82-.088 1.283-.637.463-.549.444-1.338.402-1.373zm-3.18 1.947c-.268.028-1.125.206-1.642 1.03a2.709 2.709 0 0 0-.379 1.126 4.455 4.455 0 0 0 .018 1.127c.058.402.166.78.306 1.127.157.385.353.732.56 1.026.46.65.981 1.026 1.274 1.033.604.014 1.128-.393 1.56-.383h.039c.433-.01.959.397 1.563.383.293-.007.813-.383 1.273-1.033.208-.294.402-.64.559-1.026.023-.057.046-.114.068-.173-.45-.141-.82-.493-1.02-.954a1.963 1.963 0 0 1-.13-1.127 1.746 1.746 0 0 1 .906-1.25c-.484-.762-1.272-.88-1.52-.906-.539-.056-1.298.362-1.71.37h-.012c-.413-.008-1.172-.426-1.711-.37z" fill="{color_ios}" fillRule="evenodd" />"#
                    ),
                    vault_repo: format!(
                        r#"<path d="M12.3519074,20 L15.6797981,20 L16.0095931,22.214392 C16.188924,23.4185001 15.2692105,24.464 14.0602801,24.464 L13.9714254,24.464 C12.7554258,24.464 11.844822,23.4047996 12.0221124,22.214392 L12.3519074,20 Z M14.817,20.999 L13.213,20.999 L13.0112031,22.3616998 C12.929477,22.9104461 13.3113046,23.3950919 13.8530094,23.4572725 L13.9714254,23.464 L14.0602801,23.464 C14.624945,23.464 15.0489506,23.0202599 15.0311816,22.4790411 L15.0205024,22.3616998 L14.817,20.999 Z" fill="{color_graphics_i}" fillRule="evenodd" />"#
                    ),
                    link_bg: format!(
                        r#"<path d="M16 10v18h8.004c.55 0 .996-.447.996-.996 0-2.881.004-5.774-.004-8.649V10H16z" fill="{color_link}" />"#
                    ),
                    transfer_bg: format!(
                        r#"<path d="M16 10v18h8.004c.55 0 .996-.447.996-.996 0-2.881.004-5.774-.004-8.649V10H16z" fill="{color_transfer}" />"#
                    ),
                    error_overlay: None,
                },
                fold: format!(r#"<path d="M17 9V1h-1v9h9V9z" fill="{color_graphics_i}" />"#),
                shadow: format!(
                    r#"<path opacity=".2" d="M18 10h7v5l-7-5z" fill="{color_black}" />"#
                ),
                dl: format!(
                    r#"<path d="M20 16v9l-1.904-1.83a.502.502 0 0 0-.701.004.504.504 0 0 0-.004.703L20.501 27c1.014-1.02 2.069-2.079 3.109-3.123a.498.498 0 1 0-.703-.707L21 25c-.011-3.323 0-5.928 0-9z" fill="{color_white}" />"#
                ),
                ul: format!(
                    r#"<path d="M20 27v-9l-1.904 1.83a.502.502 0 0 1-.701-.004.504.504 0 0 1-.004-.703L20.501 16c1.014 1.02 2.069 2.079 3.109 3.123a.498.498 0 1 1-.703.707L21 18c-.011 3.323 0 5.928 0 9z" fill="{color_white}" />"#
                ),
                dl_ul: format!(
                    r#"<path d="M20.5 16l-3.11 3.123a.504.504 0 0 0 .005.703.5.5 0 0 0 .7.004L20 18v7l-1.904-1.83a.502.502 0 0 0-.701.004.504.504 0 0 0-.004.703L20.5 27l3.11-3.123a.498.498 0 1 0-.703-.707L21 25c-.002-2.41-.01-4.093 0-7l1.906 1.83a.498.498 0 0 0 .703-.707L20.5 16z" fill="{color_white}" />"#
                ),
                categories: HashMap::from([
                    (
                        FileIconCategory::Archive,
                        format!(
                            r#"<g><path d="M8 1h2v2H8V1zM6 3h2v2H6V3zm2 2h2v2H8V5zM6 7h2v2H6V7zm2 2h2v2H8V9zm-2 2h2v2H6v-2z" fill="{color_graphics_j}" /><path d="M6 15h4v2H6z" fill="{color_graphics_h}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Audio,
                        format!(
                            r#"<path d="M8.896 22.25v-9.151l.447-.062 7.672-1.063.589-.081v9.321h.018v1.367a1.36 1.36 0 0 1-1.37 1.366h-1.117a1.365 1.365 0 0 1-1.37-1.366c0-.755.602-1.367 1.37-1.367h1.433v-8.727l.589 1.549-7.672 1.063.447-1.55v8.701h.006v1.367c0 .754-.603 1.366-1.371 1.366H7.451a1.365 1.365 0 0 1-1.371-1.366c0-.755.603-1.367 1.371-1.367h1.445z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                    (
                        FileIconCategory::Code,
                        format!(
                            r#"<path d="M10.704 22.244a3180.543 3180.543 0 0 0-4.08-1.624v-1.072a3180.496 3180.496 0 0 1 4.08-1.624v1.296a42 42 0 0 1-1.096.444c-.363.141-.728.29-1.096.444a737.66 737.66 0 0 0 2.192.824v1.312zm.6.344l2.424-6.224 1.264.384-2.432 6.224-1.256-.384zm4.288-4.664a3180.59 3180.59 0 0 0 4.08 1.624v1.072a3180.59 3180.59 0 0 1-4.08 1.624v-1.312a737.636 737.636 0 0 1 2.192-.824 45.012 45.012 0 0 0-1.096-.444c-.363-.141-.728-.29-1.096-.444v-1.296z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                    (
                        FileIconCategory::Document,
                        format!(
                            r#"<g><path d="M5 14v2h16v-2H5z" fill="{color_graphics_a}" /><path d="M5 18v2h16v-2H5z" fill="{color_graphics_j}" /><path d="M5 22v2h16v-2H5z" fill="{color_graphics_j}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Image,
                        format!(
                            r#"<g><path d="M22.843 25.764v-3.85l-6.511-7.664L7 25.764z" fill="{color_graphics_e}" /><path d="M3 25.763v-2.706l5.623-5.287 7.814 7.99z" fill="{color_graphics_f}" /><circle cx="7.043" cy="14.293" r="2.043" fill="{color_graphics_g}"/></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Pdf,
                        format!(
                            r#"<path d="M11.848 12.4a.935.935 0 0 0-.694.309c-.244.278-.267.498-.222.873.023.193.059.43.101.686.185 1.948.001 3.935-.771 5.457-.065.125-.128.248-.192.367-1.966.618-3.53 1.66-3.816 2.697-.136.5-.011 1 .342 1.373.357.373.799.57 1.29.57 1.147 0 2.262-1.05 3.432-3.052a8.82 8.82 0 0 1 1.139-.196c.203-.02.515-.061.705-.091a8.484 8.484 0 0 1 1.33-.09c1.404 1.424 2.588 2.164 3.6 2.164.67 0 1.24-.338 1.574-.926.242-.434.244-.93.004-1.363-.523-.948-2.327-1.551-4.342-1.701a33.45 33.45 0 0 1-.303-.34c-1.124-1.287-1.814-3.174-2.15-5.145a13.76 13.76 0 0 0-.078-.658c-.077-.542-.299-.934-.95-.934zm.718 6.448c.135.24.277.468.424.687l-.113.018.09.592-.092-.592c-.15.023-.385.054-.549.072.09-.25.17-.51.24-.777zm6.002 3.07l.055.03-.002.001-.053-.031z" fill="{color_graphics_c}" />"#
                        ),
                    ),
                    (
                        FileIconCategory::Presentation,
                        format!(
                            r#"<g><path d="M9.057 9c-.05 0-.098 0-.147.002v4.73h4.402A4.285 4.285 0 0 0 9.056 9z" fill="{color_graphics_b}" /><path d="M7.73 10.184a4.284 4.284 0 0 0-4.13 4.285 4.283 4.283 0 0 0 4.277 4.289 4.284 4.284 0 0 0 4.256-3.846H7.73v-4.728z" fill="{color_graphics_j}" /><path d="M9 22v2h12v-2H9zm-4 0v2h2v-2H5z" fill="{color_graphics_j}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Sheet,
                        format!(
                            r#"<g><path d="M5 14v10h16V14H5zm2 2h8v2H7v-2zm10 0h2v2h-2v-2zM7 20h8v2H7v-2zm10 0h2v2h-2v-2z" fill="{color_graphics_j}" /><path d="M5 10v2h8v-2H5z" fill="{color_graphics_d}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Text,
                        format!(
                            r#"<path d="M5 14v2h16v-2H5zm0 4v2h16v-2H5zm0 4v2h16v-2H5z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                    (
                        FileIconCategory::Video,
                        format!(
                            r#"<path d="M5 13.43V26.6h16.036V13.43H5zm1.43 1.463h1.427v1.462H6.43v-1.462zm11.75 0h1.427v1.462H18.18v-1.462zm-8.895.037h7.465V25.1H9.285V14.93zM6.43 17.818h1.427v1.465H6.43v-1.465zm11.75 0h1.427v1.465H18.18v-1.465zM6.43 20.746h1.427v1.465H6.43v-1.465zm11.75 0h1.427v1.465H18.18v-1.465zM6.43 23.674h1.427v1.463H6.43v-1.463zm11.75 0h1.427v1.463H18.18v-1.463z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                ]),
                error: format!(
                    r#"<path d="M13.489 18.921l4.795 4.789a.993.993 0 001.403 0l.022-.022a.99.99 0 000-1.402l-4.795-4.789 4.79-4.782a.99.99 0 000-1.402l-.023-.023a.993.993 0 00-1.404 0l-4.79 4.783-4.77-4.765a.993.993 0 00-1.404 0l-.022.022a.99.99 0 000 1.401l4.772 4.766-4.772 4.765a.99.99 0 000 1.402l.022.022a.993.993 0 001.404 0l4.772-4.765z" fill="{color_graphics_m}" />"#
                ),
            },

            lg: FileIconDefSize {
                width: 60,
                height: 68,
                file: FileIconDefFile {
                    bg: format!(
                        r#"<path d="M59.001 19.281c0-.271-.213-.753-.41-.93L41.566 1.41c-.243-.22-.753-.41-1.081-.41H1.01C1.004 1 1 67 1 67h58.001V19.281z" fill="{color_white}" />"#
                    ),
                    border: format!(
                        r#"<path d="M59.001 19.281c0-.271-.213-.753-.41-.93L41.566 1.41c-.243-.22-.753-.41-1.081-.41H1.01C1.004 1 1 67 1 67h58.001V19.281zM42.246.668l17.013 16.943c.407.367.741 1.118.741 1.673v47.71A.995.995 0 0 1 59.009 68H.99A.999.999 0 0 1 0 67V1c0-.548.452-1 1.01-1h39.488c.57 0 1.339.3 1.748.668z" fill="{color_graphics_i}" />"#
                    ),
                    shared_border: format!(r#"<path d="M1 43h24v24H1z" fill="{color_white}" />"#),
                    export_bg: format!(r#"<path d="M1 45h22v22H1z" fill="{color_export}" />"#),
                    import_bg: format!(r#"<path d="M1 45h22v22H1z" fill="{color_import}" />"#),
                    shared: format!(
                        r#"<path d="M5 60.908a3.097 3.097 0 0 1 3.086-3.091h7.578a3.091 3.091 0 0 1 3.086 3.091V64H5v-3.092zm6.95-5.27c-2.126 0-3.85-1.71-3.85-3.819S9.824 48 11.95 48c2.126 0 3.85 1.71 3.85 3.82 0 2.108-1.724 3.818-3.85 3.818z" fill="{color_white}" />"#
                    ),
                    link_bg: format!(r#"<path d="M40 20h19v47H40z" fill="{color_link}" />"#),
                    transfer_bg: format!(
                        r#"<path d="M40 20h19v47H40z" fill="{color_transfer}" />"#
                    ),
                    error_overlay: Some(format!(
                        r#"<path d="M59.001 20H40V1H1.01C1.004 1 1 67 1 67h58.001V20z" fill="{color_white}" opacity=".7" />"#
                    )),
                },
                folder: FileIconDefFolder {
                    bg: format!(
                        r#"<path d="M7 1h33.484c.329 0 .84.191 1.082.41L58.59 18.35c.197.177.412.66.412.931v7.133H59v37.172c0 1.886-1.496 3.416-3.322 3.416H4.322C2.49 67.002 1 65.477 1 63.586V26.414c0-1.886 1.496-3.416 3.322-3.416.88.003 1.86.002 2.678 0C7.004 15.665 6.995 8.333 7 1z" fill="{color_white}" />"#
                    ),
                    border: format!(
                        r#"<path d="M40.498 0L7.014.002c-.558 0-1.01.453-1.01 1.002V22H4.322C1.94 22 0 23.982 0 26.414v37.172C0 66.023 1.931 68 4.322 68h51.356C58.06 68 60 66.018 60 63.586V19.285c0-.555-.333-1.306-.74-1.674L42.246.668C41.837.299 41.068 0 40.498 0zM7 1h33.484c.329 0 .84.191 1.082.41L58.59 18.35c.197.177.412.66.412.931v7.133H59v37.172c0 1.886-1.496 3.416-3.322 3.416H4.322C2.49 67.002 1 65.477 1 63.586V26.414c0-1.886 1.496-3.416 3.322-3.416L6 23h.004v18.438H7V22c0-9.555-.002-21 0-21z" fill="{color_graphics_i}" />"#
                    ),
                    split: FileIconDefFolderSplit {
                        normal: format!(
                            r#"<path d="M0 42h8.382c2.252 0 5.18-1.436 6.61-3.18l2.068-2.523C17.942 35.222 19.833 34 21.225 34h38.687v-1H21.225c-1.692 0-3.867 1.356-4.938 2.663l-2.069 2.523C12.978 39.698 10.334 41 8.382 41H0v1z" fill="{color_graphics_i}" />"#
                        ),
                        export: def_lg_folder_split(color_export),
                        import: def_lg_folder_split(color_import),
                        android: def_lg_folder_split(color_android_split),
                        ios: def_lg_folder_split(color_ios_split),
                    },
                    export: format!(
                        r#"<path d="M24.95 44c-2.127 0-3.85 1.71-3.85 3.818 0 2.11 1.723 3.82 3.85 3.82 2.126 0 3.85-1.71 3.85-3.82C28.8 45.71 27.077 44 24.95 44zm16.2 0c-2.126 0-3.85 1.71-3.85 3.818 0 2.11 1.724 3.82 3.85 3.82 2.127 0 3.85-1.71 3.85-3.82C45 45.71 43.277 44 41.15 44zm-20.064 9.816A3.097 3.097 0 0 0 18 56.908V60h13.75v-3.092a3.091 3.091 0 0 0-3.086-3.092h-7.578zm16.25 0a3.097 3.097 0 0 0-3.086 3.092V60H48v-3.092a3.091 3.091 0 0 0-3.086-3.092h-7.578z" fill="{color_export}" />"#
                    ),
                    import: format!(
                        r#"<path d="M24.95 44c-2.127 0-3.85 1.71-3.85 3.818 0 2.11 1.723 3.82 3.85 3.82 2.126 0 3.85-1.71 3.85-3.82C28.8 45.71 27.077 44 24.95 44zm16.2 0c-2.126 0-3.85 1.71-3.85 3.818 0 2.11 1.724 3.82 3.85 3.82 2.127 0 3.85-1.71 3.85-3.82C45 45.71 43.277 44 41.15 44zm-20.064 9.816A3.097 3.097 0 0 0 18 56.908V60h13.75v-3.092a3.091 3.091 0 0 0-3.086-3.092h-7.578zm16.25 0a3.097 3.097 0 0 0-3.086 3.092V60H48v-3.092a3.091 3.091 0 0 0-3.086-3.092h-7.578z" fill="{color_import}" />"#
                    ),
                    android: format!(
                        r#"<path d="M45.11 48.49c-.014-.019-.057-.012-.094.014-.038.027-.057.065-.043.084l.652.912c-1.893.834-1.953 2.855-1.953 2.855h7.676s-.084-2.012-1.975-2.85l.658-.917c.014-.02-.007-.057-.045-.084-.037-.026-.078-.033-.091-.014l-.676.946c-.463-.18-1.024-.29-1.71-.291-.694-.003-1.262.106-1.728.285l-.672-.94zm.703 1.77c.258 0 .466.205.466.459s-.208.46-.467.46a.464.464 0 0 1-.466-.46c0-.254.208-.46.467-.46zm3.378 0c.258 0 .467.205.467.459s-.209.46-.467.46a.466.466 0 0 1-.468-.46c0-.254.21-.46.468-.46zm-6.795 2.29a.817.817 0 0 0-.822.811v3.643c0 .448.369.81.822.81a.817.817 0 0 0 .823-.81V53.36a.817.817 0 0 0-.823-.81zm10.196 0a.817.817 0 0 0-.822.811v3.643c0 .448.368.81.822.81a.817.817 0 0 0 .822-.81V53.36a.817.817 0 0 0-.822-.81zm-8.951.259v6.421a.46.46 0 0 0 .463.456h.984v2.023c0 .448.368.81.822.81a.817.817 0 0 0 .822-.81v-2.023h1.524v2.023c0 .448.368.81.822.81a.817.817 0 0 0 .822-.81v-2.023h.987c.254 0 .46-.204.46-.456v-3.285h.014V52.81h-7.72z" fill="{color_android}" fillRule="evenodd" />"#
                    ),
                    ios: format!(
                        r#"<path d="M50.186 48.488c-.07-.058-1.346.177-2.137 1.06-.792.885-.744 2.232-.674 2.29.07.059 1.367-.146 2.139-1.06.772-.915.741-2.231.672-2.29zm-4.948 3.244a1.89 1.89 0 0 0-.351.002c-.447.047-1.877.343-2.739 1.715-.302.482-.534 1.097-.63 1.88a6.828 6.828 0 0 0-.047.913c.004.329.03.651.076.965.096.67.277 1.299.512 1.877.26.641.585 1.22.931 1.709.767 1.084 1.636 1.712 2.125 1.723 1.006.022 1.879-.659 2.6-.641a.077.077 0 0 0 .023.002h.018c.007 0 .016 0 .023-.002.721-.018 1.596.663 2.602.64.489-.01 1.356-.638 2.123-1.722a8.559 8.559 0 0 0 1.043-2c-.749-.235-1.365-.819-1.697-1.586a3.278 3.278 0 0 1-.217-1.879 2.908 2.908 0 0 1 1.172-1.879c.107-.076.22-.144.336-.205-.807-1.271-2.12-1.467-2.534-1.51-.898-.093-2.164.602-2.851.614h-.018c-.601-.01-1.646-.544-2.5-.616z" fill="{color_ios}" />"#
                    ),
                    vault_repo: format!(
                        r#"<path d="M30.6109504,48 L36.3885383,48 L36.9610991,51.9684444 C37.2724374,54.1263443 35.6757127,56 33.5768751,56 L33.4226136,56 C31.3115031,56 29.7305938,54.1017915 30.0383896,51.9684444 L30.6109504,48 Z M35.522,49 L31.476,49 L31.0281412,52.1112442 C30.8135437,53.5986294 31.8540721,54.9035867 33.2644302,54.9949008 L33.4226136,55 L33.5768751,55 C35.0152451,55 36.1144262,53.7577579 35.9902268,52.276734 L35.9713475,52.1112442 L35.522,49 Z" fill="{color_graphics_i}" fillRule="evenodd" />"#
                    ),
                    link_bg: format!(
                        r#"<path d="M40 20v47h15.695C57.513 66.99 59 65.465 59 63.586V20H40z" fill="{color_link}" />"#
                    ),
                    transfer_bg: format!(
                        r#"<path d="M40 20v47h15.695C57.513 66.99 59 65.465 59 63.586V20H40z" fill="{color_transfer}" />"#
                    ),
                    error_overlay: None,
                },
                fold: format!(r#"<path d="M41 1h-1v19h19v-1H41z" fill="{color_graphics_i}" />"#),
                shadow: format!(
                    r#"<path opacity=".2" d="M44 20h15v11L44 20z" fill="{color_black}" />"#
                ),
                dl: format!(
                    r#"<path d="M49 47v17l-1.904-1.83a.502.502 0 0 0-.701.004.504.504 0 0 0-.004.703L49.5 66c1.014-1.02 2.069-2.079 3.109-3.123a.498.498 0 1 0-.703-.707L50 64c-.012-3.323 0-13.928 0-17z" fill="{color_white}" />"#
                ),
                ul: format!(
                    r#"<path d="M49.5 47l-3.11 3.123a.504.504 0 0 0 .005.703.5.5 0 0 0 .7.004L49 49v17h1c0-3.072-.011-13.677 0-17l1.906 1.83a.498.498 0 0 0 .703-.707L49.5 47z" fill="{color_white}" />"#
                ),
                dl_ul: format!(
                    r#"<path d="M49.5 47l-3.11 3.123a.504.504 0 0 0 .005.703.5.5 0 0 0 .7.004L49 49v15l-1.904-1.83a.502.502 0 0 0-.701.004.504.504 0 0 0-.004.703L49.5 66l3.11-3.123a.498.498 0 1 0-.703-.707L50 64c-.001-3.62-.01-11.893 0-15l1.906 1.83a.498.498 0 0 0 .703-.707L49.5 47z" fill="{color_white}" />"#
                ),
                categories: HashMap::from([
                    (
                        FileIconCategory::Archive,
                        format!(
                            r#"<g><g fill="{color_graphics_j}"><path d="M13.286 1.18h2.529v2.529h-2.529z" /><path d="M15.814 3.709h2.529v2.529h-2.529z" /><path d="M13.286 6.237h2.529v2.529h-2.529z" /><path d="M15.814 8.766h2.529v2.529h-2.529z" /><path d="M13.286 11.294h2.529v2.529h-2.529z" /><path d="M15.814 13.823h2.529v2.529h-2.529z" /><path d="M13.286 16.351h2.529v2.529h-2.529z" /><path d="M15.814 18.88h2.529v2.529h-2.529z" /><path d="M13.286 21.409h2.529v2.529h-2.529z" /><path d="M15.814 23.937h2.529v2.529h-2.529z" /><path d="M13.286 26.466h2.529v2.529h-2.529z" /><path d="M15.814 28.994h2.529v2.529h-2.529z" /><path d="M13.286 31.523h2.529v2.529h-2.529z" /><path d="M15.814 34.051h2.529v2.529h-2.529z" /><path d="M13.286 36.58h2.529v2.529h-2.529z" /></g><path d="M13.286 41.637h5.057v2.529h-5.057z" fill="{color_graphics_h}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Audio,
                        format!(
                            r#"<path d="M40.5 28.426l-.568.078-17 2.355-.432.06v19.53h-3.838a3.01 3.01 0 0 0-3.012 3.008 3.013 3.013 0 0 0 3.012 3.01h1.824a3.013 3.013 0 0 0 3.014-3.01V34.789l16-2.215v15.942h-3.838a3.01 3.01 0 0 0-3.012 3.007 3.011 3.011 0 0 0 3.012 3.008h1.826a3.01 3.01 0 0 0 3.012-3.008V28.426z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                    (
                        FileIconCategory::Code,
                        format!(
                            r#"<path d="M8.874 47.605v-1.944l13.662-6.075v2.97l-9.126 4.077 9.126 4.077v2.97L8.874 47.605zm17.82 8.505l-2.24-.81 8.423-20.952 2.241.864-8.424 20.898zm24.327-10.449v1.944L37.36 53.68v-2.97l9.126-4.077-9.126-4.077v-2.97l13.662 6.075z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                    (
                        FileIconCategory::Document,
                        format!(
                            r#"<g><path d="M6 30.395v2.5h48v-2.5H6zM6 14.72v2.5h18.1v-2.5H6zm0 5.323v2.5h29.9v-2.5H6z" fill="{color_graphics_j}" /><path d="M6 9.396v2.5h18.1v-2.5H6z" fill="{color_graphics_a}" /><path d="M6 25.07v2.5h48v-2.5H6zm0 10.352v2.5h48v-2.5H6z" fill="{color_graphics_j}" /><path d="M6 40.744v2.5h29.9v-2.5H6z" fill="{color_graphics_a}" /><path d="M6 51.393v2.5h32.9v-2.5H6zm0 5.322v2.5h29.8v-2.5H6zm0-10.647v2.5h48v-2.5H6z" fill="{color_graphics_j}" /><path d="M42.5 53.096l2.613 9.205h2.21l1.82-6.229h.039l1.82 6.229h2.197l2.625-9.205h-2.312l-1.483 6.123h-.039l-1.65-6.123h-2.315l-1.638 6.123h-.04l-1.454-6.123H42.5z" fill="{color_graphics_a}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Image,
                        format!(
                            r#"<g><path d="M54.06 60.723v-8.896L39.036 34.122 17.5 60.722z" fill="{color_graphics_e}" /><path d="M6 60.72v-6.25l12.976-12.215 18.032 18.46z" fill="{color_graphics_f}" /><ellipse cx="15.952" cy="34.221" rx="4.714" ry="4.719" fill="{color_graphics_g}"/></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Pdf,
                        format!(
                            r#"<g><path d="M6 30.395v2.5h48v-2.5H6zM6 14.72v2.5h29.4v-2.5H6zm0 5.323v2.5h29.4v-2.5H6z" fill="{color_graphics_j}" /><path d="M6 9.396v2.5h17.6v-2.5H6zM6 25.07v2.5h17.6v-2.5H6z" fill="{color_graphics_c}" /><path d="M6 35.414v2.5h48v-2.5H6zm0 5.33v2.5h48v-2.5H6zm0 10.649v2.5h35.3v-2.5H6zm0 5.322v2.5h29.4v-2.5H6zm0-10.647v2.5h48v-2.5H6z" fill="{color_graphics_j}" /><path d="M47.527 51.541a.935.935 0 0 0-.693.309c-.244.278-.266.495-.22.87.023.193.056.433.099.688.185 1.949.003 3.935-.77 5.457-.064.126-.13.248-.193.367-1.966.618-3.531 1.66-3.816 2.698a1.41 1.41 0 0 0 .341 1.37c.357.373.801.573 1.293.573 1.146 0 2.26-1.05 3.43-3.053.391-.092.774-.16 1.14-.197.204-.021.514-.06.704-.09a8.484 8.484 0 0 1 1.33-.09c1.404 1.425 2.588 2.162 3.6 2.162.67 0 1.24-.337 1.574-.925.242-.434.243-.929.004-1.362v-.002h-.002c-.525-.946-2.327-1.549-4.34-1.699a33.485 33.485 0 0 1-.303-.34c-1.124-1.287-1.813-3.173-2.148-5.144-.02-.215-.05-.449-.08-.658-.077-.543-.299-.934-.95-.934zm.721 6.445c.134.24.274.47.422.69l-.113.015.091.594-.093-.592c-.15.024-.385.055-.55.073.09-.251.173-.512.243-.78zm6 3.073l.055.029-.002.002-.053-.031zm-10.68.949h.004l-.004.002v-.002z" fill="{color_graphics_c}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Presentation,
                        format!(
                            r#"<g><path d="M19.176 9.11a9.62 9.62 0 0 0-.295.005v9.32h8.805a8.36 8.36 0 0 0 .044-.872c0-4.668-3.83-8.454-8.554-8.454z" fill="{color_graphics_b}" /><path d="M16.52 11.44c-4.588.153-8.26 3.877-8.26 8.447 0 4.668 3.83 8.453 8.554 8.453 4.426 0 8.067-3.32 8.51-7.578H16.52v-9.323z" fill="{color_graphics_j}" /><path d="M10.809 40.502v2.5H54v-2.5H10.809zM6 40.506v2.5h2.5v-2.5H6zm0 5.224v2.5h2.5v-2.5H6zm4.809.016v2.5H54v-2.5H10.809zM6 50.951v2.5h2.5v-2.5H6zm4.809.041v2.5h30.789v-2.5h-30.79z" fill="{color_graphics_j}" /><path d="M52.43 55.214c0 .537-.1.99-.299 1.359" fill="{color_graphics_b}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Sheet,
                        format!(
                            r#"<g><path d="M6 14.72v2.5h27.68v-2.5H6zm0 5.57v28.3h48v-28.3H6zm2.5 2.5h7.55v2.28H8.5v-2.28zm10.05 0h22.7v2.28h-22.7v-2.28zm25.2 0h7.75v2.28h-7.75v-2.28zM8.5 27.57h7.55v2.825H8.5V27.57zm10.05 0h22.7v2.825h-22.7V27.57zm25.2 0h7.75v2.825h-7.75V27.57zM8.5 32.895h7.55v2.527H8.5v-2.527zm10.05 0h22.7v2.527h-22.7v-2.527zm25.2 0h7.75v2.527h-7.75v-2.527zM8.5 37.922h7.55v2.822H8.5v-2.822zm10.05 0h22.7v2.822h-22.7v-2.822zm25.2 0h7.75v2.822h-7.75v-2.822zM8.5 43.244h7.55v2.846H8.5v-2.846zm10.05 0h22.7v2.846h-22.7v-2.846zm25.2 0h7.75v2.846h-7.75v-2.846zM6 51.393v2.5h36.2v-2.5H6zm0 5.322v2.5h29.3v-2.5H6z" fill="{color_graphics_j}" /><path d="M44.855 52.797l2.965 4.342L44.596 62h2.652l1.938-3.25L51.199 62h2.744l-3.328-4.861 2.912-4.342H50.94l-1.689 2.86-1.717-2.86h-2.678z" fill="{color_graphics_d}" /><path d="M6 9.396v2.5h17.6v-2.5H6z" fill="{color_graphics_d}" /></g>"#
                        ),
                    ),
                    (
                        FileIconCategory::Text,
                        format!(
                            r#"<path d="M6 9.277v2.5h27.68v-2.5zm0 5.246v2.5h27.68v-2.5zm0 5.247v2.5h32.131v-2.5zm0 4.955v2.5h41.373v-2.5zm0 5.246v2.5h29.4v-2.5zm0 4.953v2.5h48v-2.5zm0 5.246v2.5h48v-2.5zm0 5.248v2.5h48v-2.5zm0 5.244v2.5h35.3v-2.5zm0 5.248v2.5h29.4v-2.5z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                    (
                        FileIconCategory::Video,
                        format!(
                            r#"<path d="M14.5 27v32.5h29V27h-29zm2.5 2.5h2.5V32H17v-2.5zm5 0h14v12.55H22V29.5zm16.5 0H41V32h-2.5v-2.5zm-21.5 5h2.5V37H17v-2.5zm21.5 0H41V37h-2.5v-2.5zm-21.5 5h2.5V42H17v-2.5zm21.5 0H41V42h-2.5v-2.5zm-21.5 5h2.5V47H17v-2.5zm21.5 0H41V47h-2.5v-2.5zm-16.5.05h14V57H22V44.55zm-5 4.95h2.5V52H17v-2.5zm21.5 0H41V52h-2.5v-2.5zm-21.5 5h2.5V57H17v-2.5zm21.5 0H41V57h-2.5v-2.5z" fill="{color_graphics_j}" />"#
                        ),
                    ),
                ]),

                error: format!(
                    r#"<path d="M45.542 55.075a1.49 1.49 0 01-2.12-.037c-7.638-7.975-20.156-8.02-27.851-.102a1.49 1.49 0 01-2.121.02 1.523 1.523 0 01-.021-2.14c8.877-9.134 23.34-9.081 32.15.119.575.601.559 1.56-.037 2.14z" fill="{color_graphics_m}" />"#
                ),
            },
        }
    }
}

pub struct FileIconFactory {
    defs: FileIconDefs,
}

impl FileIconFactory {
    pub fn new(theme: &FileIconTheme) -> Self {
        let defs = FileIconDefs::generate(theme);

        Self { defs }
    }

    pub fn generate_svg(&self, props: &FileIconProps) -> (String, u32, u32) {
        let FileIconProps { size, attrs, .. } = props;
        let category = &attrs.category;
        let FileIconAttrs {
            is_dl,
            is_ul,
            is_download_transfer,
            is_upload_transfer,
            is_export,
            is_import,
            is_android,
            is_ios,
            is_vault_repo,
            is_error,
            ..
        } = *attrs;

        let is_folder = matches!(category, FileIconCategory::Folder);

        let def = match size {
            FileIconSize::Sm => &self.defs.sm,
            FileIconSize::Lg => &self.defs.lg,
        };

        let mut svg = String::new();

        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            def.width, def.height, def.width, def.height,
        ));

        if is_folder {
            svg.push_str(&def.folder.bg);
            svg.push_str(&def.folder.border);
        } else {
            svg.push_str(&def.file.bg);
            svg.push_str(&def.file.border);
        }

        if is_folder {
            if is_export {
                svg.push_str(&def.folder.split.export);
            } else if is_import {
                svg.push_str(&def.folder.split.import);
            } else if is_android {
                svg.push_str(&def.folder.split.android);
            } else if is_ios {
                svg.push_str(&def.folder.split.ios);
            } else {
                svg.push_str(&def.folder.split.normal);
            }
        }

        svg.push_str(&def.fold);

        if is_folder {
            // is_export and is_import must be last
            if is_android {
                svg.push_str(&def.folder.android);
            } else if is_ios {
                svg.push_str(&def.folder.ios);
            } else if is_vault_repo {
                svg.push_str(&def.folder.vault_repo);
            } else if is_export {
                svg.push_str(&def.folder.export);
            } else if is_import {
                svg.push_str(&def.folder.import);
            }
        } else {
            if let Some(content) = def.categories.get(category) {
                svg.push_str(content);
            }
        }

        if is_dl || is_ul {
            if is_folder {
                svg.push_str(&def.folder.link_bg);
            } else {
                svg.push_str(&def.file.link_bg);
            }

            if is_dl && is_ul {
                svg.push_str(&def.dl_ul);
            } else if is_dl {
                svg.push_str(&def.dl);
            } else if is_ul {
                svg.push_str(&def.ul);
            }
        }

        if is_download_transfer || is_upload_transfer {
            if is_folder {
                svg.push_str(&def.folder.transfer_bg);
            } else {
                svg.push_str(&def.file.transfer_bg);
            }

            if is_download_transfer && is_upload_transfer {
                svg.push_str(&def.dl_ul);
            } else if is_download_transfer {
                svg.push_str(&def.dl);
            } else if is_upload_transfer {
                svg.push_str(&def.ul);
            }
        }

        svg.push_str(&def.shadow);

        if !is_folder {
            if is_export || is_import {
                svg.push_str(&def.file.shared_border);

                if is_export {
                    svg.push_str(&def.file.export_bg);
                } else if is_import {
                    svg.push_str(&def.file.import_bg);
                }

                svg.push_str(&def.file.shared);
            }
        }

        if is_error {
            if is_folder {
                if let Some(error_overlay) = &def.folder.error_overlay {
                    svg.push_str(&error_overlay);
                }
            } else {
                if let Some(error_overlay) = &def.file.error_overlay {
                    svg.push_str(&error_overlay);
                }
            }

            svg.push_str(&def.error);
        }

        svg.push_str("</svg>");

        (svg, def.width, def.height)
    }
}
