/// 宿主版本号，来源于 Cargo 包版本。
pub const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 当前宿主支持的包 API 版本。
pub const HOST_API_VERSION: u32 = 1;

/// 当前宿主支持的 package.json 清单版本。
pub const PACKAGE_MANIFEST_VERSION: u32 = 1;

/// 当前宿主写入并读取的截屏/录屏清单版本。
pub const MEDIA_MANIFEST_VERSION: u32 = 1;

/// 图片字符画缓存格式版本。
pub const IMAGE_CACHE_FORMAT_VERSION: u8 = 2;
