use phf::phf_map;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileIconType {
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

static EXT_TO_FILE_ICON_TYPE: phf::Map<&'static str, FileIconType> = phf_map! {
    "3ds" => FileIconType::Image,
    "3g2" => FileIconType::Video,
    "3ga" => FileIconType::Video,
    "3gp" => FileIconType::Video,
    "3gp2" => FileIconType::Video,
    "3gpp" => FileIconType::Video,
    "3gpp2" => FileIconType::Video,
    "7z" => FileIconType::Archive,
    "aac" => FileIconType::Audio,
    "ac3" => FileIconType::Audio,
    "adb" => FileIconType::Text,
    "ads" => FileIconType::Text,
    "ag" => FileIconType::Image,
    "aif" => FileIconType::Audio,
    "aifc" => FileIconType::Audio,
    "aiff" => FileIconType::Audio,
    "aiffc" => FileIconType::Audio,
    "amr" => FileIconType::Audio,
    "amz" => FileIconType::Audio,
    "ape" => FileIconType::Audio,
    "arj" => FileIconType::Archive,
    "art" => FileIconType::Image,
    "arw" => FileIconType::Image,
    "asc" => FileIconType::Text,
    "asf" => FileIconType::Video,
    "asp" => FileIconType::Code,
    "aspx" => FileIconType::Code,
    "ass" => FileIconType::Text,
    "asx" => FileIconType::Video,
    "au" => FileIconType::Audio,
    "avf" => FileIconType::Video,
    "avi" => FileIconType::Video,
    "awb" => FileIconType::Audio,
    "axa" => FileIconType::Audio,
    "axv" => FileIconType::Video,
    "bat" => FileIconType::Text,
    "bdm" => FileIconType::Video,
    "bdmv" => FileIconType::Video,
    "bib" => FileIconType::Text,
    "bmp" => FileIconType::Image,
    "boo" => FileIconType::Text,
    "brf" => FileIconType::Text,
    "c" => FileIconType::Code,
    "c++" => FileIconType::Code,
    "cbl" => FileIconType::Text,
    "cc" => FileIconType::Text,
    "cdr" => FileIconType::Image,
    "cdt" => FileIconType::Image,
    "cgm" => FileIconType::Image,
    "class" => FileIconType::Code,
    "clpi" => FileIconType::Video,
    "cls" => FileIconType::Text,
    "cmake" => FileIconType::Text,
    "cob" => FileIconType::Text,
    "cpi" => FileIconType::Video,
    "cpp" => FileIconType::Code,
    "cpt" => FileIconType::Image,
    "cr2" => FileIconType::Image,
    "crw" => FileIconType::Image,
    "cs" => FileIconType::Text,
    "csd" => FileIconType::Audio,
    "csh" => FileIconType::Text,
    "css" => FileIconType::Code,
    "csv" => FileIconType::Text,
    "csvs" => FileIconType::Text,
    "cur" => FileIconType::Image,
    "cxx" => FileIconType::Text,
    "d" => FileIconType::Text,
    "dart" => FileIconType::Code,
    "dcl" => FileIconType::Text,
    "dcr" => FileIconType::Image,
    "dds" => FileIconType::Image,
    "deb" => FileIconType::Archive,
    "di" => FileIconType::Text,
    "dif" => FileIconType::Video,
    "diff" => FileIconType::Text,
    "divx" => FileIconType::Video,
    "djv" => FileIconType::Image,
    "djvu" => FileIconType::Image,
    "dl" => FileIconType::Video,
    "dng" => FileIconType::Image,
    "doc" => FileIconType::Document,
    "docx" => FileIconType::Document,
    "dot" => FileIconType::Document,
    "dotx" => FileIconType::Document,
    "dsl" => FileIconType::Text,
    "dts" => FileIconType::Audio,
    "dtshd" => FileIconType::Audio,
    "dtx" => FileIconType::Text,
    "dv" => FileIconType::Video,
    "dwg" => FileIconType::Image,
    "dxf" => FileIconType::Image,
    "e" => FileIconType::Text,
    "eif" => FileIconType::Text,
    "el" => FileIconType::Text,
    "emf" => FileIconType::Image,
    "eps" => FileIconType::Image,
    "epsf" => FileIconType::Image,
    "epsi" => FileIconType::Image,
    "erf" => FileIconType::Image,
    "erl" => FileIconType::Text,
    "etx" => FileIconType::Text,
    "exr" => FileIconType::Image,
    "f" => FileIconType::Text,
    "f4a" => FileIconType::Audio,
    "f4b" => FileIconType::Audio,
    "f4v" => FileIconType::Video,
    "f90" => FileIconType::Text,
    "f95" => FileIconType::Text,
    "fig" => FileIconType::Image,
    "fits" => FileIconType::Image,
    "flac" => FileIconType::Audio,
    "flc" => FileIconType::Video,
    "fli" => FileIconType::Video,
    "flv" => FileIconType::Video,
    "fo" => FileIconType::Text,
    "for" => FileIconType::Text,
    "fxm" => FileIconType::Video,
    "g3" => FileIconType::Image,
    "gcd" => FileIconType::Text,
    "gcrd" => FileIconType::Text,
    "gem" => FileIconType::Archive,
    "gif" => FileIconType::Image,
    "gl" => FileIconType::Video,
    "go" => FileIconType::Code,
    "gs" => FileIconType::Text,
    "gsm" => FileIconType::Audio,
    "gtar" => FileIconType::Archive,
    "gv" => FileIconType::Text,
    "gvp" => FileIconType::Text,
    "gz" => FileIconType::Archive,
    "h" => FileIconType::Code,
    "h++" => FileIconType::Text,
    "hh" => FileIconType::Text,
    "hp" => FileIconType::Text,
    "hpp" => FileIconType::Text,
    "hs" => FileIconType::Text,
    "htc" => FileIconType::Text,
    "htm" => FileIconType::Code,
    "html" => FileIconType::Code,
    "hxx" => FileIconType::Text,
    "icb" => FileIconType::Image,
    "icns" => FileIconType::Image,
    "ico" => FileIconType::Image,
    "ics" => FileIconType::Text,
    "icz" => FileIconType::Text,
    "idl" => FileIconType::Text,
    "ief" => FileIconType::Image,
    "iff" => FileIconType::Image,
    "ilbm" => FileIconType::Image,
    "ime" => FileIconType::Text,
    "imy" => FileIconType::Text,
    "ins" => FileIconType::Text,
    "iptables" => FileIconType::Text,
    "it" => FileIconType::Audio,
    "jad" => FileIconType::Text,
    "java" => FileIconType::Code,
    "jng" => FileIconType::Image,
    "jp2" => FileIconType::Image,
    "jpe" => FileIconType::Image,
    "jpeg" => FileIconType::Image,
    "jpf" => FileIconType::Image,
    "jpg" => FileIconType::Image,
    "jpg2" => FileIconType::Image,
    "jpm" => FileIconType::Image,
    "jpx" => FileIconType::Image,
    "js" => FileIconType::Code,
    "jsm" => FileIconType::Code,
    "json" => FileIconType::Code,
    "jsp" => FileIconType::Code,
    "jsx" => FileIconType::Code,
    "k25" => FileIconType::Image,
    "kar" => FileIconType::Audio,
    "kdc" => FileIconType::Image,
    "key" => FileIconType::Presentation,
    "ksh" => FileIconType::Text,
    "latex" => FileIconType::Text,
    "lbm" => FileIconType::Image,
    "ldif" => FileIconType::Text,
    "less" => FileIconType::Code,
    "lhs" => FileIconType::Text,
    "log" => FileIconType::Text,
    "lrv" => FileIconType::Video,
    "lsf" => FileIconType::Video,
    "lsx" => FileIconType::Video,
    "ltx" => FileIconType::Text,
    "lua" => FileIconType::Text,
    "lwo" => FileIconType::Image,
    "lwob" => FileIconType::Image,
    "lws" => FileIconType::Image,
    "ly" => FileIconType::Text,
    "lz" => FileIconType::Archive,
    "m" => FileIconType::Text,
    "m15" => FileIconType::Audio,
    "m1u" => FileIconType::Video,
    "m1v" => FileIconType::Video,
    "m2t" => FileIconType::Video,
    "m2ts" => FileIconType::Video,
    "m3u" => FileIconType::Audio,
    "m3u8" => FileIconType::Audio,
    "m4a" => FileIconType::Audio,
    "m4b" => FileIconType::Audio,
    "m4u" => FileIconType::Video,
    "m4v" => FileIconType::Video,
    "mak" => FileIconType::Text,
    "manifest" => FileIconType::Text,
    "markdown" => FileIconType::Text,
    "md" => FileIconType::Text,
    "mdi" => FileIconType::Image,
    "me" => FileIconType::Text,
    "med" => FileIconType::Audio,
    "mid" => FileIconType::Audio,
    "midi" => FileIconType::Audio,
    "minipsf" => FileIconType::Audio,
    "mk" => FileIconType::Text,
    "mk3d" => FileIconType::Video,
    "mka" => FileIconType::Audio,
    "mkd" => FileIconType::Text,
    "mkv" => FileIconType::Video,
    "ml" => FileIconType::Text,
    "mli" => FileIconType::Text,
    "mm" => FileIconType::Text,
    "mml" => FileIconType::Text,
    "mng" => FileIconType::Video,
    "mo" => FileIconType::Text,
    "mo3" => FileIconType::Audio,
    "moc" => FileIconType::Text,
    "mod" => FileIconType::Audio,
    "mof" => FileIconType::Text,
    "moov" => FileIconType::Video,
    "mov" => FileIconType::Video,
    "movie" => FileIconType::Video,
    "mp2" => FileIconType::Video,
    "mp3" => FileIconType::Audio,
    "mp4" => FileIconType::Video,
    "mpa" => FileIconType::Video,
    "mpc" => FileIconType::Audio,
    "mpe" => FileIconType::Video,
    "mpeg" => FileIconType::Video,
    "mpega" => FileIconType::Audio,
    "mpg" => FileIconType::Video,
    "mpga" => FileIconType::Audio,
    "mpl" => FileIconType::Video,
    "mpls" => FileIconType::Video,
    "mpp" => FileIconType::Audio,
    "mpv" => FileIconType::Video,
    "mrl" => FileIconType::Text,
    "mrml" => FileIconType::Text,
    "mrw" => FileIconType::Image,
    "ms" => FileIconType::Text,
    "msod" => FileIconType::Image,
    "mtm" => FileIconType::Audio,
    "mts" => FileIconType::Video,
    "mup" => FileIconType::Text,
    "mxu" => FileIconType::Video,
    "nef" => FileIconType::Image,
    "nfo" => FileIconType::Text,
    "not" => FileIconType::Text,
    "nsv" => FileIconType::Video,
    "ocl" => FileIconType::Text,
    "odp" => FileIconType::Presentation,
    "ods" => FileIconType::Sheet,
    "odt" => FileIconType::Document,
    "oga" => FileIconType::Audio,
    "ogg" => FileIconType::Audio,
    "ogm" => FileIconType::Video,
    "ogv" => FileIconType::Video,
    "ooc" => FileIconType::Text,
    "opml" => FileIconType::Text,
    "opus" => FileIconType::Audio,
    "ora" => FileIconType::Image,
    "orc" => FileIconType::Audio,
    "orf" => FileIconType::Image,
    "p" => FileIconType::Text,
    "pas" => FileIconType::Text,
    "pat" => FileIconType::Image,
    "patch" => FileIconType::Text,
    "pbm" => FileIconType::Image,
    "pcd" => FileIconType::Image,
    "pct" => FileIconType::Image,
    "pcx" => FileIconType::Image,
    "pdf" => FileIconType::Pdf,
    "pef" => FileIconType::Image,
    "pgm" => FileIconType::Image,
    "php" => FileIconType::Code,
    "pic" => FileIconType::Image,
    "pict" => FileIconType::Image,
    "pict1" => FileIconType::Image,
    "pict2" => FileIconType::Image,
    "pkg" => FileIconType::Archive,
    "pl" => FileIconType::Code,
    "pla" => FileIconType::Audio,
    "pls" => FileIconType::Audio,
    "pm" => FileIconType::Text,
    "png" => FileIconType::Image,
    "pnm" => FileIconType::Image,
    "pntg" => FileIconType::Image,
    "po" => FileIconType::Text,
    "pot" => FileIconType::Presentation,
    "potx" => FileIconType::Presentation,
    "ppa" => FileIconType::Presentation,
    "ppm" => FileIconType::Image,
    "pps" => FileIconType::Presentation,
    "ppsx" => FileIconType::Presentation,
    "ppt" => FileIconType::Presentation,
    "pptx" => FileIconType::Presentation,
    "ppz" => FileIconType::Presentation,
    "psd" => FileIconType::Image,
    "psf" => FileIconType::Audio,
    "psflib" => FileIconType::Audio,
    "psid" => FileIconType::Audio,
    "pwz" => FileIconType::Presentation,
    "py" => FileIconType::Code,
    "pyx" => FileIconType::Code,
    "qif" => FileIconType::Image,
    "qml" => FileIconType::Text,
    "qmlproject" => FileIconType::Text,
    "qmltypes" => FileIconType::Text,
    "qt" => FileIconType::Video,
    "qtif" => FileIconType::Image,
    "qtvr" => FileIconType::Video,
    "ra" => FileIconType::Audio,
    "raf" => FileIconType::Image,
    "ram" => FileIconType::Audio,
    "rar" => FileIconType::Archive,
    "ras" => FileIconType::Image,
    "raw" => FileIconType::Image,
    "rax" => FileIconType::Audio,
    "rb" => FileIconType::Code,
    "reg" => FileIconType::Text,
    "rej" => FileIconType::Text,
    "rgb" => FileIconType::Image,
    "rle" => FileIconType::Image,
    "rm" => FileIconType::Audio,
    "roff" => FileIconType::Text,
    "rp" => FileIconType::Image,
    "rpm" => FileIconType::Archive,
    "rs" => FileIconType::Code,
    "rss" => FileIconType::Code,
    "rt" => FileIconType::Text,
    "rtf" => FileIconType::Text,
    "rtx" => FileIconType::Text,
    "rv" => FileIconType::Video,
    "rvx" => FileIconType::Video,
    "rw2" => FileIconType::Image,
    "s3m" => FileIconType::Audio,
    "sass" => FileIconType::Code,
    "scala" => FileIconType::Text,
    "scm" => FileIconType::Text,
    "sco" => FileIconType::Audio,
    "scss" => FileIconType::Code,
    "sct" => FileIconType::Text,
    "sd2" => FileIconType::Audio,
    "sfv" => FileIconType::Text,
    "sgi" => FileIconType::Image,
    "sgm" => FileIconType::Text,
    "sgml" => FileIconType::Text,
    "sh" => FileIconType::Code,
    "shtml" => FileIconType::Code,
    "sid" => FileIconType::Audio,
    "sk" => FileIconType::Image,
    "sk1" => FileIconType::Image,
    "sldx" => FileIconType::Presentation,
    "slk" => FileIconType::Text,
    "snd" => FileIconType::Audio,
    "spec" => FileIconType::Text,
    "spx" => FileIconType::Audio,
    "sql" => FileIconType::Code,
    "sr2" => FileIconType::Image,
    "srf" => FileIconType::Image,
    "srt" => FileIconType::Text,
    "ss" => FileIconType::Text,
    "ssa" => FileIconType::Text,
    "stm" => FileIconType::Audio,
    "sty" => FileIconType::Text,
    "sub" => FileIconType::Text,
    "sun" => FileIconType::Image,
    "sv" => FileIconType::Text,
    "svg" => FileIconType::Image,
    "svgz" => FileIconType::Image,
    "svh" => FileIconType::Text,
    "swift" => FileIconType::Code,
    "sylk" => FileIconType::Text,
    "t" => FileIconType::Text,
    "t2t" => FileIconType::Text,
    "tar" => FileIconType::Archive,
    "tcl" => FileIconType::Text,
    "tex" => FileIconType::Text,
    "texi" => FileIconType::Text,
    "texinfo" => FileIconType::Text,
    "text" => FileIconType::Text,
    "tga" => FileIconType::Image,
    "tif" => FileIconType::Image,
    "tiff" => FileIconType::Image,
    "tk" => FileIconType::Text,
    "tm" => FileIconType::Text,
    "toml" => FileIconType::Code,
    "tpic" => FileIconType::Image,
    "tr" => FileIconType::Text,
    "ts" => FileIconType::Code,
    "tsv" => FileIconType::Text,
    "tsx" => FileIconType::Code,
    "tta" => FileIconType::Audio,
    "ttl" => FileIconType::Text,
    "txt" => FileIconType::Text,
    "uil" => FileIconType::Text,
    "uls" => FileIconType::Text,
    "ult" => FileIconType::Audio,
    "uni" => FileIconType::Audio,
    "uue" => FileIconType::Text,
    "v" => FileIconType::Text,
    "vala" => FileIconType::Text,
    "vapi" => FileIconType::Text,
    "vb" => FileIconType::Code,
    "vcard" => FileIconType::Text,
    "vcf" => FileIconType::Text,
    "vcs" => FileIconType::Text,
    "vct" => FileIconType::Text,
    "vda" => FileIconType::Image,
    "vhd" => FileIconType::Text,
    "vhdl" => FileIconType::Text,
    "viv" => FileIconType::Video,
    "vivo" => FileIconType::Video,
    "vlc" => FileIconType::Audio,
    "vob" => FileIconType::Video,
    "voc" => FileIconType::Audio,
    "vst" => FileIconType::Image,
    "vtt" => FileIconType::Text,
    "wav" => FileIconType::Audio,
    "wax" => FileIconType::Audio,
    "wbmp" => FileIconType::Image,
    "webm" => FileIconType::Video,
    "webp" => FileIconType::Image,
    "wiz" => FileIconType::Document,
    "wm" => FileIconType::Video,
    "wma" => FileIconType::Audio,
    "wmf" => FileIconType::Image,
    "wml" => FileIconType::Text,
    "wmls" => FileIconType::Text,
    "wmv" => FileIconType::Video,
    "wmx" => FileIconType::Video,
    "wsc" => FileIconType::Text,
    "wsgi" => FileIconType::Text,
    "wv" => FileIconType::Audio,
    "wvc" => FileIconType::Audio,
    "wvp" => FileIconType::Audio,
    "wvx" => FileIconType::Video,
    "x3f" => FileIconType::Image,
    "xbm" => FileIconType::Image,
    "xcf" => FileIconType::Image,
    "xi" => FileIconType::Audio,
    "xla" => FileIconType::Sheet,
    "xlam" => FileIconType::Sheet,
    "xlb" => FileIconType::Sheet,
    "xlc" => FileIconType::Sheet,
    "xld" => FileIconType::Sheet,
    "xll" => FileIconType::Sheet,
    "xlm" => FileIconType::Sheet,
    "xls" => FileIconType::Sheet,
    "xlsb" => FileIconType::Sheet,
    "xlsm" => FileIconType::Sheet,
    "xlsx" => FileIconType::Sheet,
    "xlt" => FileIconType::Sheet,
    "xltx" => FileIconType::Sheet,
    "xlw" => FileIconType::Sheet,
    "xm" => FileIconType::Audio,
    "xmf" => FileIconType::Audio,
    "xmi" => FileIconType::Text,
    "xml" => FileIconType::Code,
    "xpm" => FileIconType::Image,
    "xslfo" => FileIconType::Text,
    "xul" => FileIconType::Text,
    "xwd" => FileIconType::Image,
    "yaml" => FileIconType::Code,
    "z" => FileIconType::Archive,
    "zip" => FileIconType::Archive,
};

pub fn ext_to_file_icon_type(ext_lower: &str) -> Option<FileIconType> {
    EXT_TO_FILE_ICON_TYPE.get(ext_lower).cloned()
}
