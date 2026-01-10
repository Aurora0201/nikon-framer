// src/utils/payloadHelpers.js

/**
 * 构建符合 Rust ExportConfig 结构的导出配置
 * @param {Object} settings - store.exportSettings
 * @returns {Object} Rust ExportConfig
 */
export function buildExportPayload(settings) {
  return {
    // 逻辑转换：只有自定义模式才传路径，否则传 null
    targetDir: settings.pathMode === 'custom' ? settings.customPath : null,
    format: settings.format,
    // 确保转换为整数，防止滑块传字符串
    quality: parseInt(settings.quality) || 90 
  };
}

/**
 * 构建符合 Rust StyleOptions 结构的样式配置
 * 处理默认值回退和类型转换
 * * @param {String} styleId - 当前选中的样式 ID
 * @param {Object} userParams - store.modeParams (用户输入的参数)
 * @param {Object} registry - frameRegistry (用于查找默认配置)
 * @returns {Object} Rust StyleOptions (flattened)
 */
export function buildStylePayload(styleId, userParams, registry) {
  // 1. 基础 Payload
  const payload = { 
    style: styleId 
  };

  // 2. 获取配置定义以读取默认值
  const config = registry.get(styleId);

  // 3. 动态参数注入 & 类型安全转换 (OCP 核心)
  if (config && config.defaultParams) {
    Object.keys(config.defaultParams).forEach(key => {
      const defaultValue = config.defaultParams[key];
      const userValue = userParams[key];

      // A. 如果用户没填，回退到默认值
      if (userValue === undefined || userValue === null) {
        payload[key] = defaultValue;
        return;
      }

      // B. 智能类型推断 (防止 HTML input 返回字符串导致 Rust 解析失败)
      const expectedType = typeof defaultValue;
      if (expectedType === 'number') {
        const parsed = parseFloat(userValue);
        payload[key] = isNaN(parsed) ? defaultValue : parsed;
      } 
      else if (expectedType === 'boolean') {
        payload[key] = Boolean(userValue);
      } 
      else {
        payload[key] = userValue;
      }
    });
  }

  return payload;
}