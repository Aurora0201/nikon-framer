use super::models::{ParsedImageContext, RawExifData};

/// 品牌解析器特征 (Interface)
/// 任何想接入系统的品牌 (Nikon, Sony, etc.) 都必须实现这个 Trait
pub trait BrandParser: Send + Sync {
    
    /// 1. 职责链检查：判断当前解析器是否能处理这份数据
    /// 例如：NikonParser 会检查 raw.make 是否包含 "NIKON"
    fn can_parse(&self, raw: &RawExifData) -> bool;

    /// 2. 核心逻辑：执行清洗
    /// 输入原始脏数据，输出完美的上下文结构体
    fn parse(&self, raw: &RawExifData) -> ParsedImageContext;
}