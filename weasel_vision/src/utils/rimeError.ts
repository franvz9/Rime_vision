/**
 * RimeError 结构化错误类型
 * 对应 Rust 端 RimeError 的 serde 序列化格式：
 * { code: "Io", detail: "文件操作失败: ..." }
 */
export interface RimeError {
  code: string
  detail: string
}

/**
 * 从 Tauri invoke 错误中提取 RimeError
 * Tauri command 返回 Result<T, RimeError> 时，前端收到的错误格式为 { code, detail }
 */
export function isRimeError(err: unknown): err is RimeError {
  return typeof err === 'object' && err !== null && 'code' in err && 'detail' in err
}

export function getErrorCategory(err: unknown): string {
  if (!isRimeError(err)) return 'unknown'
  switch (err.code) {
    case 'NotFound':
      return 'config'
    case 'Permission':
      return 'permission'
    case 'Parse':
      return 'parse'
    case 'InvalidPath':
      return 'path'
    case 'Config':
      return 'config'
    case 'Io':
      return 'io'
    case 'RateLimited':
      return 'rate'
    default:
      return 'unknown'
  }
}

export function getErrorMessage(err: unknown): string {
  if (err === null || err === undefined) return '未知错误'
  if (isRimeError(err)) return err.detail
  if (err instanceof Error) return err.message
  if (typeof err === 'string') return err
  return String(err)
}
