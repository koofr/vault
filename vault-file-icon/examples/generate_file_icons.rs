use std::{fs, path::PathBuf};

use vault_file_icon::{
    FileIconAttrs, FileIconCategory, FileIconFactory, FileIconProps, FileIconSize, FileIconTheme,
};

fn main() {
    let theme = FileIconTheme::default();
    let factory = FileIconFactory::new(&theme);

    let svg_path: PathBuf = ["gen", "svg"].iter().collect();
    #[cfg(feature = "render")]
    let png_path: PathBuf = ["gen", "png"].iter().collect();

    fs::create_dir_all(&svg_path).unwrap();
    #[cfg(feature = "render")]
    fs::create_dir_all(&png_path).unwrap();

    for (i, props) in generate_all_props().iter().enumerate() {
        let name = format!("file-icon-{:0>3}-{}", i, props_name(props));

        let (svg, width, height) = factory.generate_svg(props);
        let svg_name = format!("{}.svg", name);
        fs::write(svg_path.clone().join(svg_name), &svg).unwrap();

        let _ = width;
        let _ = height;

        #[cfg(feature = "render")]
        {
            let png = vault_file_icon::render_png(&svg, width * 2, height * 2).unwrap();
            let png_name = format!("{}@2x.png", name);
            fs::write(png_path.clone().join(png_name), png).unwrap();
        }
    }
}

fn generate_all_props() -> Vec<FileIconProps> {
    let mut v = vec![];

    let sizes = [FileIconSize::Sm, FileIconSize::Lg];
    let categories = [
        FileIconCategory::Generic,
        FileIconCategory::Folder,
        FileIconCategory::Archive,
        FileIconCategory::Audio,
        FileIconCategory::Code,
        FileIconCategory::Document,
        FileIconCategory::Image,
        FileIconCategory::Pdf,
        FileIconCategory::Presentation,
        FileIconCategory::Sheet,
        FileIconCategory::Text,
        FileIconCategory::Video,
    ];
    let dls = [false, true];
    let uls = [false, true];
    let download_transfers = [false, true];
    let upload_transfers = [false, true];
    let export_imports = [(false, false), (true, false), (false, true)];
    let android_ios_vault_repos = [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ];
    let errors = [false, true];

    for size in &sizes {
        for category in &categories {
            for is_dl in dls {
                for is_ul in uls {
                    if is_ul && !matches!(category, FileIconCategory::Folder) {
                        continue;
                    }
                    for is_download_transfer in download_transfers {
                        for is_upload_transfer in upload_transfers {
                            if is_dl || is_ul {
                                continue;
                            }
                            for (is_export, is_import) in export_imports {
                                for (is_android, is_ios, is_vault_repo) in android_ios_vault_repos {
                                    if (is_android || is_ios || is_vault_repo)
                                        && !matches!(category, FileIconCategory::Folder)
                                    {
                                        continue;
                                    }

                                    for is_error in errors {
                                        v.push(FileIconProps {
                                            size: size.clone(),
                                            attrs: FileIconAttrs {
                                                category: category.clone(),
                                                is_dl,
                                                is_ul,
                                                is_download_transfer,
                                                is_upload_transfer,
                                                is_export,
                                                is_import,
                                                is_ios,
                                                is_android,
                                                is_vault_repo,
                                                is_error,
                                            },
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    v
}

fn props_name(props: &FileIconProps) -> String {
    let bool_display = |b: bool| if b { "t" } else { "f" };

    format!(
        "size={}-category={}-dl={}-ul={}-export={}-import={}-android={}-ios={}-error={}",
        format!("{:?}", props.size).to_lowercase(),
        format!("{:?}", props.attrs.category).to_lowercase(),
        bool_display(props.attrs.is_dl),
        bool_display(props.attrs.is_ul),
        bool_display(props.attrs.is_export),
        bool_display(props.attrs.is_import),
        bool_display(props.attrs.is_android),
        bool_display(props.attrs.is_ios),
        bool_display(props.attrs.is_error)
    )
}
