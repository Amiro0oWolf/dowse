//! 图片文件判定（[`is_image`]）、OCR 原始输出清洗（[`clean_cjk_spaces`]/
//! [`dual_form_content`]），以及 Tesseract 执行和资源发现逻辑。

use std::path::Path;

/// 支持 OCR 的图片扩展名（不含点，小写）。见设计文档第三节"索引侧变化"。
const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp"];

/// 单张图片体积上限，沿用文本抽取的 20MB 预算（extract.rs::MAX_FILE_BYTES 同值，
/// 两边独立定义是因为图片和文本走的是两条完全不同的管线，没必要为了共享一个常量
/// 硬把 extract.rs 的私有常量拉成 pub(crate)）。
pub(crate) const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

/// 这个文件是否落在 OCR 管线的处理范围内（只看扩展名，不读文件，跟
/// extract.rs::is_extractable 是同款设计：启动对账拿它做便宜的预过滤）。
pub(crate) fn is_image(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    IMAGE_EXTS.contains(&ext.to_ascii_lowercase().as_str())
}

/// 一个字符是否算 CJK 表意文字。只覆盖常用汉字区、扩展 A 区、兼容表意文字区——
/// 清洗规则只关心"两侧是不是汉字"，没必要连生僻的补充平面区块也考虑进来。
fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0xF900..=0xFAFF)
}

/// 清洗 OCR 原始输出：OCR 引擎有时会在每个词/字之间插一个空格，包括中文
/// 单字之间（实测 `test_zh.png` 识别结果是 "分 布 式 限 流 器 ..."，逐字带空格）。
/// 只删"两侧都是 CJK 字符"的空格，把它还原成 "分布式限流器"；英文单词间的空格
/// （"milestone 4 spike"）必须保留，删了英文就连成一坨读不出来、也搜不到了。
pub(crate) fn clean_cjk_spaces(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        if c == ' ' {
            let prev = out.chars().last();
            let next = chars.get(i + 1).copied();
            if let (Some(p), Some(n)) = (prev, next)
                && is_cjk(p)
                && is_cjk(n)
            {
                continue;
            }
        }
        out.push(c);
    }
    out
}

/// 双形态入索引内容：OCR 原始输出 + 去 CJK 间空格拼接后的形态一起写进 content，
/// 用空间换召回——左右结构汉字偶发拆字（池→氵也）之类的系统性误差没法自动纠正，
/// 两种形态哪个能命中都算搜到。清洗后跟原文一模一样（纯英文场景，没有 CJK 间空格
/// 可删）时不重复堆一份，省一点索引体积。
///
/// 空输入（识别不出任何文字）返回空字符串，调用方按"已处理但无文字"记录，不重试。
pub(crate) fn dual_form_content(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    let cleaned = clean_cjk_spaces(raw);
    if cleaned == raw {
        raw.to_string()
    } else {
        format!("{raw}\n{cleaned}")
    }
}

mod engine_impl {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::OnceLock;

    static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

    fn tesseract_command(executable: &Path) -> Command {
        let mut command = Command::new(executable);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            use windows::Win32::System::Threading::{
                BELOW_NORMAL_PRIORITY_CLASS, CREATE_NO_WINDOW,
            };

            command.creation_flags((BELOW_NORMAL_PRIORITY_CLASS | CREATE_NO_WINDOW).0);
        }
        command
    }

    pub fn set_resource_dir(path: PathBuf) -> anyhow::Result<()> {
        RESOURCE_DIR
            .set(path)
            .map_err(|_| anyhow::anyhow!("Tesseract resource directory is already configured"))
    }

    pub fn is_available() -> bool {
        let Some((executable, tessdata)) = find_installation() else {
            return false;
        };
        let Ok(output) = tesseract_command(&executable)
            .arg("--list-langs")
            .arg("--tessdata-dir")
            .arg(tessdata)
            .output()
        else {
            return false;
        };
        output.status.success() && {
            let languages = String::from_utf8_lossy(&output.stdout);
            ["eng", "fas"]
                .iter()
                .all(|required| languages.lines().any(|line| line.trim() == *required))
        }
    }

    fn find_installation() -> Option<(PathBuf, PathBuf)> {
        let mut directories = Vec::new();
        if let Some(directory) = RESOURCE_DIR.get() {
            directories.push(directory.clone());
        }
        if let Ok(executable) = std::env::current_exe()
            && let Some(parent) = executable.parent()
        {
            directories.push(parent.join("resources").join("tesseract"));
            directories.push(parent.join("tesseract"));
        }
        if let Some(prefix) = std::env::var_os("TESSDATA_PREFIX") {
            let prefix = PathBuf::from(prefix);
            if prefix.file_name().is_some_and(|name| name == "tessdata") {
                if let Some(parent) = prefix.parent() {
                    directories.push(parent.to_path_buf());
                }
            } else {
                directories.push(prefix);
            }
        }
        if let Some(executable) = std::env::var_os("TESSERACT_CMD") {
            let executable = PathBuf::from(executable);
            if let Some(parent) = executable.parent() {
                directories.push(parent.to_path_buf());
            }
        }
        if let Some(path) = std::env::var_os("PATH") {
            directories.extend(std::env::split_paths(&path));
        }

        directories.into_iter().find_map(|directory| {
            let executable = directory.join("tesseract.exe");
            let executable = if executable.is_file() {
                executable
            } else {
                let executable = directory.join("tesseract");
                if !executable.is_file() {
                    return None;
                }
                executable
            };
            let tessdata = if directory.file_name().is_some_and(|name| name == "tessdata") {
                directory.clone()
            } else {
                directory.join("tessdata")
            };
            (tessdata.join("eng.traineddata").is_file()
                && tessdata.join("fas.traineddata").is_file())
            .then_some((executable, tessdata))
        })
    }

    pub struct Engine {
        executable: PathBuf,
        tessdata: PathBuf,
    }

    pub fn create_engine() -> anyhow::Result<Engine> {
        let (executable, tessdata) = find_installation().ok_or_else(|| {
            anyhow::anyhow!(
                "bundled Tesseract OCR or required English/Persian traineddata was not found"
            )
        })?;
        Ok(Engine {
            executable,
            tessdata,
        })
    }

    fn strip_extended_prefix(path: &Path) -> PathBuf {
        PathBuf::from(crate::display_path(&path.to_string_lossy()))
    }

    pub fn recognize(engine: &Engine, path: &Path) -> anyhow::Result<String> {
        let output = tesseract_command(&engine.executable)
            .arg(strip_extended_prefix(path))
            .arg("stdout")
            .arg("-l")
            .arg("eng+fas")
            .arg("--tessdata-dir")
            .arg(&engine.tessdata)
            .output()
            .map_err(|error| {
                anyhow::anyhow!("start bundled Tesseract for {}: {error}", path.display())
            })?;
        if !output.status.success() {
            anyhow::bail!(
                "Tesseract failed for {}: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn strip_extended_prefix_removes_drive_prefix() {
            assert_eq!(
                strip_extended_prefix(Path::new(r"\\?\C:\a\b.png")),
                PathBuf::from(r"C:\a\b.png")
            );
        }

        #[test]
        fn strip_extended_prefix_converts_unc_form() {
            assert_eq!(
                strip_extended_prefix(Path::new(r"\\?\UNC\server\share\a.png")),
                PathBuf::from(r"\\server\share\a.png")
            );
        }

        #[test]
        fn strip_extended_prefix_leaves_normal_paths_untouched() {
            assert_eq!(
                strip_extended_prefix(Path::new(r"C:\a\b.png")),
                PathBuf::from(r"C:\a\b.png")
            );
        }
    }
}

/// 检查 Tesseract 与内置英语、波斯语模型是否可用。
pub fn is_available() -> bool {
    engine_impl::is_available()
}

pub use engine_impl::set_resource_dir;
pub(crate) use engine_impl::{create_engine, recognize};

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn is_image_matches_supported_extensions_case_insensitively() {
        assert!(is_image(Path::new("shot.png")));
        assert!(is_image(Path::new("shot.PNG")));
        assert!(is_image(Path::new("photo.JPEG")));
        assert!(is_image(Path::new("a.webp")));
        assert!(is_image(Path::new("a.bmp")));
        assert!(!is_image(Path::new("note.txt")));
        assert!(!is_image(Path::new("noext")));
    }

    /// 纯中文场景：OCR 逐字输出、字间带空格（实测 test_zh.png 的真实样本），
    /// 清洗后应该完全贴回不带空格的连续汉字。
    #[test]
    fn clean_cjk_spaces_removes_spaces_between_cjk_characters() {
        let raw = "分 布 式 限 流 器 的 令 牌 桶 实 现";
        assert_eq!(clean_cjk_spaces(raw), "分布式限流器的令牌桶实现");
    }

    /// 中英混排场景（实测 test_mixed.png 的真实样本）：CJK-CJK 间的空格删掉，
    /// 英文单词之间、以及英文和 CJK 交界处的空格必须保留，否则英文单词会连成一坨。
    #[test]
    fn clean_cjk_spaces_keeps_spaces_touching_latin_text() {
        let raw = "Rust 调 用 Windows.Media.Ocr API OcrEngine.TryCreateFromLanguage(zh-Hans) 截 图 文 字 进 索 引 milestone 4 spike test";
        let cleaned = clean_cjk_spaces(raw);
        assert_eq!(
            cleaned,
            "Rust 调用 Windows.Media.Ocr API OcrEngine.TryCreateFromLanguage(zh-Hans) 截图文字进索引 milestone 4 spike test"
        );
    }

    /// 纯英文场景：不存在 CJK-CJK 间的空格，清洗应该是恒等变换。
    #[test]
    fn clean_cjk_spaces_is_identity_for_pure_latin_text() {
        let raw = "milestone 4 spike test with multiple english words";
        assert_eq!(clean_cjk_spaces(raw), raw);
    }

    #[test]
    fn dual_form_content_stacks_raw_and_cleaned_when_they_differ() {
        let raw = "分 布 式 限 流 器";
        let content = dual_form_content(raw);
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines, vec!["分 布 式 限 流 器", "分布式限流器"]);
    }

    #[test]
    fn dual_form_content_does_not_duplicate_when_cleaning_is_a_no_op() {
        let raw = "milestone 4 spike test";
        assert_eq!(dual_form_content(raw), raw);
    }

    #[test]
    fn dual_form_content_of_blank_text_is_empty() {
        assert_eq!(dual_form_content("   \n  "), "");
        assert_eq!(dual_form_content(""), "");
    }
}
