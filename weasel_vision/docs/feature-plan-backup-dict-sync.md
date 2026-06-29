# WeaselVision 备份、词典管理与同步功能设计

## 一、功能全景

```
备份、词典与同步管理
├── 配置备份管理
│   ├── 自动备份（每次写入配置时创建）
│   ├── 手动备份（一键打包全部配置）
│   ├── 备份列表（查看所有历史备份）
│   ├── 预览差异（对比当前配置与备份）
│   └── 回滚恢复（恢复到指定备份）
│
├── 用户词典管理
│   ├── 词典列表（已有 userdb 概览）
│   ├── 词频管理（浏览/搜索/编辑/删除词条）
│   ├── 快照管理（备份/恢复/合并词典快照）
│   └── 词典维护（清空/删除/导出/导入）
│
└── 同步管理
    ├── 同步设置（配置 sync_dir 和 installation_id）
    ├── 同步状态（当前设备标识、上次同步时间）
    ├── 设备列表（已同步的设备概览）
    ├── 执行同步（触发上传/下载/合并）
    └── 同步日志（查看同步历史和结果）
```

---

## 二、配置备份管理设计

### 2.1 备份层级

| 层级 | 内容 | 触发时机 | 存储位置 |
|------|------|---------|---------|
| **自动备份** | 单个被修改的文件 | 每次 save_patch | `backups/auto/*.yaml.时间戳.bak` |
| **手动备份** | 全部配置文件 + 用户设置 | 用户手动触发 | `backups/manual/时间戳/` |
| **部署前备份** | 全部配置文件 | 点击「重新部署」前 | `backups/deploy/时间戳/` |

### 2.2 备份目录结构

```
~/Library/Rime/                              # macOS 用户目录
├── *.custom.yaml                            # 配置文件（根目录）
│
├── backups/                                 # 所有备份统一存放
│   ├── auto/                                # 自动备份（每次写入时创建）
│   │   ├── squirrel.custom.yaml.20240115-143022.bak
│   │   ├── squirrel.custom.yaml.20240115-140000.bak
│   │   └── default.custom.yaml.20240115-143022.bak
│   │
│   ├── manual/                              # 手动备份（一键打包）
│   │   ├── 20240115-143022/
│   │   │   ├── manifest.json                # 备份元数据
│   │   │   ├── squirrel.custom.yaml
│   │   │   ├── default.custom.yaml
│   │   │   ├── installation.yaml
│   │   │   ├── user.yaml
│   │   │   └── *.schema.custom.yaml         # 方案级自定义
│   │   └── 20240114-100000/
│   │       └── ...
│   │
│   └── deploy/                              # 部署前备份
│       └── 20240115-142800/
│           ├── manifest.json
│           └── *.custom.yaml
│
└── user_dictionaries/                       # 词典快照
    ├── luna_pinyin.userdb.txt
    └── terra_pinyin.userdb.txt
```

Windows 路径对应为 `%AppData%\Rime\`。

### 2.3 manifest.json 格式

```json
{
  "id": "20240115-143022",
  "created_at": "2024-01-15T14:30:22+08:00",
  "type": "manual",
  "platform": "macos",
  "files": [
    {
      "name": "squirrel.custom.yaml",
      "size": 2048,
      "modified": "2024-01-15T14:28:00+08:00"
    },
    {
      "name": "default.custom.yaml",
      "size": 1024,
      "modified": "2024-01-15T13:00:00+08:00"
    }
  ],
  "schemas": ["luna_pinyin", "terra_pinyin"],
  "user_dict_snapshots": ["luna_pinyin.userdb.txt"],
  "note": ""
}
```

### 2.4 差异对比

备份对比采用逐行文本 diff，展示当前配置与备份的差异：

```
┌─────────────────────────────────────────────────────────┐
│ 对比: squirrel.custom.yaml                              │
│ 当前版本 vs 备份 20240115-143022                         │
├─────────────────────────────────────────────────────────┤
│ - color_scheme: native                                  │  ← 备份中的值
│ + color_scheme: merald                                  │  ← 当前值（已修改）
│                                                         │
│   inline_preedit: true                                  │  ← 无变化
│                                                         │
│ - font_point: 16                                        │
│ + font_point: 18                                        │
└─────────────────────────────────────────────────────────┘
```

### 2.5 Rust 后端命令

```rust
// 备份操作
#[tauri::command]
fn create_backup(note: Option<String>) -> Result<BackupInfo, String>

#[tauri::command]
fn list_backups() -> Result<Vec<BackupInfo>, String>

#[tauri::command]
fn get_backup_detail(backup_id: &str) -> Result<BackupDetail, String>

#[tauri::command]
fn restore_backup(backup_id: &str, restore_files: Vec<String>) -> Result<(), String>

#[tauri::command]
fn compare_backup(backup_id: &str, file_name: &str) -> Result<FileDiff, String>

#[tauri::command]
fn delete_backup(backup_id: &str) -> Result<(), String>

// 部署前自动备份
#[tauri::command]
fn deploy_with_backup() -> Result<(), String>
```

### 2.6 数据模型

```rust
#[derive(Serialize, Deserialize)]
struct BackupInfo {
    id: String,              // "20240115-143022"
    created_at: String,      // ISO 8601
    backup_type: BackupType, // manual | auto | deploy
    file_count: usize,
    total_size: i64,
    note: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct BackupDetail {
    info: BackupInfo,
    files: Vec<BackupFile>,
    schemas: Vec<String>,
    user_dict_snapshots: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct BackupFile {
    name: String,
    size: i64,
    modified: String,
    content_preview: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct FileDiff {
    file_name: String,
    current: Option<String>,
    backup: String,
    changes: Vec<DiffLine>,
}

#[derive(Serialize, Deserialize)]
struct DiffLine {
    line_number: usize,
    old: Option<String>,
    new: Option<String>,
    change_type: String,  // added | removed | unchanged
}
```

### 2.7 Vue 组件

```
BackupManager.vue              # 备份管理主页面
├── BackupList.vue             # 备份列表（按时间倒序）
├── BackupDetail.vue           # 备份详情（文件列表 + 操作）
├── BackupCompare.vue          # 差异对比（逐行 diff）
└── RestoreDialog.vue          # 恢复确认弹窗（选择要恢复的文件）
```

---

## 三、用户词典管理设计

### 3.1 词典列表

```
┌─────────────────────────────────────────────────────────┐
│ 用户词典                                                │
├──────────────┬──────────┬──────────┬──────────┬─────────┤
│ 词典名称      │ 关联方案  │ 词条数    │ 文件大小  │ 更新时间 │
├──────────────┼──────────┼──────────┼──────────┼─────────┤
│ luna_pinyin   │ 朙月拼音  │ 12,345   │ 2.3 MB   │ 今天    │
│ terra_pinyin  │ 地球拼音  │ 8,789    │ 1.8 MB   │ 昨天    │
│ double_pinyin │ 自然双拼  │ 3,456    │ 0.8 MB   │ 3天前   │
└──────────────┴──────────┴──────────┴──────────┴─────────┘
```

### 3.2 词频管理

```
┌─────────────────────────────────────────────────────────────┐
│ 词典: luna_pinyin                    🔍 [搜索编码或文字]     │
├─────────────────────────────────────────────────────────────┤
│ 排序: [词频↓] [最近使用↓] [文字↑]    筛选: [全部] [高频] [低频] │
├──────┬──────────┬──────────────┬──────┬──────────┬──────────┤
│  #   │ 文字      │ 编码          │ 词频  │ 最近使用  │ 操作     │
├──────┼──────────┼──────────────┼──────┼──────────┼──────────┤
│  1   │ 的        │ de5           │ 8923 │ 10分钟前  │ [编辑]   │
│  2   │ 是        │ shi4          │ 7654 │ 15分钟前  │ [编辑]   │
│  3   │ 我        │ wo3           │ 6543 │ 20分钟前  │ [编辑]   │
│  4   │ 你好      │ ni3 hao3      │ 5432 │ 1小时前   │ [编辑]   │
│  ... │ ...       │ ...           │ ...  │ ...      │ ...     │
├──────┴──────────┴──────────────┴──────┴──────────┴──────────┤
│ 共 12,345 词条    总词频 456,789     [批量删除] [导出] [清空]  │
│ 第 1/124 页    [< 上一页]  [下一页 >]                        │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 快照管理

```
┌─────────────────────────────────────────────────────────┐
│ luna_pinyin 词典快照                                     │
├─────────────────────────────────────────────────────────┤
│ [创建快照]  [导入外部快照...]                              │
├──────────────┬──────────┬──────────┬─────────────────────┤
│ 快照文件      │ 创建时间  │ 大小     │ 操作                 │
├──────────────┼──────────┼──────────┼─────────────────────┤
│ auto-自动     │ 今天 14:30│ 45 KB   │ [恢复] [合并] [删除] │
│ manual-手动   │ 今天 10:00│ 43 KB   │ [恢复] [合并] [删除] │
│ 外部快照      │ 昨天 22:00│ 41 KB   │ [合并] [删除]       │
└──────────────┴──────────┴──────────┴─────────────────────┘
```

### 3.4 词典操作说明

| 操作 | 说明 | 是否需要关闭输入法 |
|------|------|------------------|
| 创建快照 | `rime_dict_manager --backup` | 是 |
| 恢复快照 | `rime_dict_manager --merge` | 是 |
| 合并快照 | 从外部快照合并到本地词典 | 是 |
| 清空词典 | `rime_dict_manager --drop` | 是 |
| 导出码表 | `rime_dict_manager --export` | 是 |
| 导入码表 | `rime_dict_manager --import` | 是 |
| 浏览词条 | 解析快照文件（纯文本） | 否 |
| 搜索词条 | 解析快照文件（纯文本） | 否 |
| 编辑词频 | 修改快照文件中的频次值 | 是 |
| 删除词条 | 从快照文件中移除行 | 是 |

### 3.5 快照文件格式

```
# luna_pinyin.userdb.txt（Tab 分隔）
# 文字    编码                    词频    上屏次数    最近使用
的        de5                     8923    8923        1705312222
是        shi4                    7654    7654        1705312100
我        wo3                     6543    6543        1705312000
你好      ni3 hao3                5432    5432        1705311000
输入法    shu1 ru4 fa3            4321    4321        1705310000
```

### 3.6 Rust 后端命令

```rust
// 词典列表
#[tauri::command]
fn list_user_dictionaries() -> Result<Vec<UserDictInfo>, String>

// 词频管理
#[tauri::command]
fn load_user_dict_entries(
    dict_id: &str,
    page: usize,
    per_page: usize,
    sort_by: &str,
    search: Option<&str>,
) -> Result<DictEntriesResult, String>

#[tauri::command]
fn update_entry_frequency(
    dict_id: &str,
    word: &str,
    code: &str,
    new_freq: i64,
) -> Result<(), String>

#[tauri::command]
fn delete_entries(
    dict_id: &str,
    entries: Vec<DictEntryKey>,
) -> Result<(), String>

#[tauri::command]
fn batch_delete_low_frequency(
    dict_id: &str,
    threshold: i64,
) -> Result<i64, String>

// 快照管理
#[tauri::command]
fn list_snapshots(dict_id: &str) -> Result<Vec<SnapshotInfo>, String>

#[tauri::command]
fn create_snapshot(dict_id: &str) -> Result<(), String>

#[tauri::command]
fn restore_snapshot(
    dict_id: &str,
    snapshot_path: &str,
) -> Result<(), String>

#[tauri::command]
fn merge_snapshot(
    dict_id: &str,
    snapshot_path: &str,
) -> Result<(), String>

// 词典维护
#[tauri::command]
fn clear_user_dict(dict_id: &str) -> Result<(), String>

#[tauri::command]
fn export_user_dict(
    dict_id: &str,
    output_path: &str,
) -> Result<(), String>
```

### 3.7 数据模型

```rust
#[derive(Serialize, Deserialize)]
struct UserDictInfo {
    dict_id: String,
    display_name: String,
    schema_ids: Vec<String>,
    entry_count: usize,
    file_size: i64,
    last_modified: String,
}

#[derive(Serialize, Deserialize)]
struct DictEntry {
    word: String,
    code: String,
    frequency: i64,
    commit_count: i64,
    last_used: String,
}

#[derive(Serialize, Deserialize)]
struct DictEntriesResult {
    entries: Vec<DictEntry>,
    total: usize,
    page: usize,
    per_page: usize,
    total_frequency: i64,
}

#[derive(Serialize, Deserialize)]
struct SnapshotInfo {
    file_name: String,
    created_at: String,
    size: i64,
    snapshot_type: String,  // auto | manual | external
}

#[derive(Serialize, Deserialize)]
struct DictEntryKey {
    word: String,
    code: String,
}
```

### 3.8 Vue 组件

```
UserDictManager.vue          # 词典管理主页面
├── DictList.vue             # 词典列表
├── DictEntries.vue          # 词频管理（表格 + 搜索 + 排序 + 分页）
├── EntryEditor.vue          # 词条编辑弹窗（修改词频）
├── SnapshotManager.vue      # 快照管理
└── DictMaintenance.vue      # 词典维护（清空/删除/导出）
```

---

## 四、同步管理设计

### 4.1 同步原理

Rime 的同步**不是实时云同步**，而是通过**共享文件夹**（如 Dropbox、OneDrive、U盘）手动/半自动同步词典快照和配置文件。

```
设备 A                              共享文件夹                          设备 B
──────                             ──────────                        ──────
输入法运行                          Dropbox / U盘                      输入法运行
  ↓                                    ↓                              ↓
点击「同步」                        词典快照上传                        点击「同步」
  ↓                                    ↓                              ↓
上传词典快照 ─────────────────→ sync/id-xxx/*.userdb.txt ←─────────── 下载词典快照
上传配置文件 ─────────────────→ sync/id-xxx/*.yaml       ←─────────── 下载配置文件
  ↓                                    ↓                              ↓
合并到本地词典  ←─────────────── sync/id-yyy/*.userdb.txt ───────────→ 合并到本地词典
```

Rime 只负责读写本地文件夹，文件传输由 Dropbox/OneDrive 等客户端完成。

### 4.2 同步设置

```yaml
# installation.yaml
sync_dir: '/Users/fred/Dropbox/RimeSync'   # macOS
sync_dir: 'D:\Dropbox\RimeSync'            # Windows
installation_id: 'my-macbook'              # 设备标识（默认 UUID）
```

### 4.3 同步目录结构

```
D:\Dropbox\RimeSync\
├── my-macbook/                           # Mac 的数据
│   ├── luna_pinyin.userdb.txt            # 词典快照
│   ├── terra_pinyin.userdb.txt
│   ├── installation.yaml
│   ├── default.custom.yaml
│   └── squirrel.custom.yaml
├── my-windows/                           # Windows 的数据
│   ├── luna_pinyin.userdb.txt
│   ├── installation.yaml
│   ├── default.custom.yaml
│   └── weasel.custom.yaml
└── my-linux/                             # Linux 的数据
    ├── luna_pinyin.userdb.txt
    └── ...
```

### 4.4 同步执行流程

```
用户点击「立即同步」
    ↓
检查 sync_dir 是否配置
    ↓ (已配置)
执行同步操作:
    ├─ 1. 本地词典 → 备份为快照 *.userdb.txt
    ├─ 2. 本地快照 + 配置 → 上传到 sync_dir/当前ID/
    ├─ 3. 从 sync_dir/其他ID/ 下载快照
    └─ 4. 下载的快照 → 合并到本地词典
    ↓
更新同步状态
    ↓
显示同步结果
```

### 4.5 同步状态页面

```
┌─────────────────────────────────────────────────────────┐
│ 同步管理                                                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ 当前设备标识: my-macbook                                 │
│ 同步目录:     /Users/fred/Dropbox/RimeSync              │
│ 上次同步:     2024-01-15 14:30 (2小时前)                 │
│ 同步状态:     ✅ 已同步                                  │
│                                                         │
│ [立即同步]  [修改设置]  [打开同步文件夹]                   │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ 已同步的设备                                             │
├──────────────┬──────────────┬──────────┬────────────────┤
│ 设备标识      │ 操作系统      │ 上次同步  │ 包含词典        │
├──────────────┼──────────────┼──────────┼────────────────┤
│ my-macbook   │ macOS 14.2   │ 今天 14:30│ luna, terra    │
│ my-windows   │ Windows 11   │ 今天 12:00│ luna, double   │
│ my-linux     │ Ubuntu 22.04 │ 昨天 22:00│ luna           │
└──────────────┴──────────────┴──────────┴────────────────┘
```

### 4.6 同步设置

```
┌─────────────────────────────────────────────────────────┐
│ 同步设置                                                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│ 同步目录:                                               │
│ [/Users/fred/Dropbox/RimeSync          ] [选择...]      │
│                                                         │
│ 设备标识:                                               │
│ [my-macbook                                    ]        │
│ (建议使用小写字母、数字、横线和下划线)                      │
│                                                         │
│ 同步选项:                                               │
│ ☑ 同步用户词典快照                                       │
│ ☑ 同步配置文件                                          │
│ ☐ 同步部署缓存（build/ 目录）                             │
│                                                         │
│ [保存设置]  [恢复默认]                                   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 4.7 同步日志

```
┌─────────────────────────────────────────────────────────┐
│ 同步日志                                                │
├─────────────────────────────────────────────────────────┤
│ 2024-01-15 14:30  ✅ 同步完成                           │
│   - 上传: luna_pinyin.userdb.txt (45 KB)               │
│   - 上传: default.custom.yaml (1.2 KB)                 │
│   - 下载: my-windows/luna_pinyin.userdb.txt (42 KB)    │
│   - 合并: luna_pinyin 词频更新 234 条                    │
│                                                         │
│ 2024-01-14 10:00  ✅ 同步完成                           │
│   - 上传: luna_pinyin.userdb.txt (43 KB)               │
│   - 无新数据下载                                        │
│                                                         │
│ 2024-01-13 22:00  ⚠️ 同步部分完成                       │
│   - 上传: luna_pinyin.userdb.txt (41 KB)               │
│   - 下载失败: my-linux 目录不存在                       │
└─────────────────────────────────────────────────────────┘
```

### 4.8 合并规则

| 场景 | 处理方式 |
|------|---------|
| 同一词条在两台设备都存在 | 词频取**最大值**，其他参数叠加 |
| 词条只在一台设备存在 | 直接合并过来 |
| 配置文件冲突 | **不自动合并**，只备份，需手动处理 |

### 4.9 Rust 后端命令

```rust
#[tauri::command]
fn get_sync_settings() -> Result<SyncSettings, String>

#[tauri::command]
fn save_sync_settings(settings: SyncSettings) -> Result<(), String>

#[tauri::command]
fn get_sync_status() -> Result<SyncStatus, String>

#[tauri::command]
fn list_synced_devices() -> Result<Vec<SyncedDevice>, String>

#[tauri::command]
fn execute_sync() -> Result<SyncResult, String>

#[tauri::command]
fn get_sync_log(limit: usize) -> Result<Vec<SyncLogEntry>, String>
```

### 4.10 数据模型

```rust
#[derive(Serialize, Deserialize)]
struct SyncSettings {
    sync_dir: Option<String>,
    installation_id: String,
    sync_user_dict: bool,
    sync_config: bool,
    sync_build_cache: bool,
}

#[derive(Serialize, Deserialize)]
struct SyncStatus {
    configured: bool,
    last_sync_time: Option<String>,
    sync_dir_exists: bool,
    current_id: String,
}

#[derive(Serialize, Deserialize)]
struct SyncedDevice {
    id: String,
    platform: String,
    last_sync: String,
    synced_dicts: Vec<String>,
    synced_configs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SyncResult {
    success: bool,
    uploaded: Vec<SyncedFile>,
    downloaded: Vec<SyncedFile>,
    merged: Vec<MergedDict>,
    errors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct SyncedFile {
    file_name: String,
    size: i64,
    source_device: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MergedDict {
    dict_id: String,
    entries_updated: usize,
    entries_added: usize,
}

#[derive(Serialize, Deserialize)]
struct SyncLogEntry {
    timestamp: String,
    level: String,
    message: String,
    details: Option<String>,
}
```

### 4.11 Vue 组件

```
SyncManager.vue              # 同步管理主页面
├── SyncStatus.vue           # 同步状态卡片
├── SyncSettings.vue         # 同步设置表单
├── SyncedDevices.vue        # 设备列表
├── SyncExecute.vue          # 同步执行按钮 + 进度
└── SyncLog.vue              # 同步日志
```

---

## 五、Vue 前端组件总览

```
src/components/
├── BackupManager.vue              # 备份管理
│   ├── BackupList.vue
│   ├── BackupDetail.vue
│   ├── BackupCompare.vue
│   └── RestoreDialog.vue
│
├── UserDictManager.vue            # 词典管理
│   ├── DictList.vue
│   ├── DictEntries.vue
│   ├── EntryEditor.vue
│   ├── SnapshotManager.vue
│   └── DictMaintenance.vue
│
├── SyncManager.vue                # 同步管理（新增）
│   ├── SyncStatus.vue
│   ├── SyncSettings.vue
│   ├── SyncedDevices.vue
│   ├── SyncExecute.vue
│   └── SyncLog.vue
│
└── SidebarView.vue                # 侧边栏（新增入口）
    └── 新增: "备份管理" "词典管理" "同步管理"
```

---

## 六、侧边栏导航更新

```
当前侧边栏:
├── 通用设置
├── 主题外观
├── 方案管理
├── 语言模型
├── 快捷键
├── 标点符号
└── 高级设置

更新后:
├── 通用设置
├── 主题外观
├── 方案管理
├── 语言模型
├── 快捷键
├── 标点符号
├── 备份管理          ← 新增
├── 词典管理          ← 新增
├── 同步管理          ← 新增
└── 高级设置
```

---

## 七、实现优先级

| 阶段 | 功能 | 工作量 | 说明 |
|------|------|--------|------|
| **P0** | 手动备份 + 备份列表 + 回滚恢复 | 3-4 天 | 配置安全基础 |
| **P1** | 差异对比 + 部署前自动备份 | 2-3 天 | 配置安全增强 |
| **P2** | 词典列表 + 快照备份/恢复 | 2-3 天 | 词典管理基础 |
| **P3** | 词频浏览 + 搜索 + 排序 | 2-3 天 | 词频管理核心 |
| **P4** | 同步设置 + 设备列表 | 1-2 天 | 同步基础 |
| **P5** | 执行同步 + 同步日志 | 2-3 天 | 同步核心 |
| **P6** | 单词条编辑/删除 + 批量操作 | 1-2 天 | 词典精细管理 |
| **P7** | 词典维护（清空/删除/导出） | 0.5 天 | 危险操作 |

预计总工作量：**15-20 天**

---

## 八、技术要点

| 模块 | 关键点 |
|------|--------|
| 备份 | manifest.json 记录元数据，支持逐行 diff 对比 |
| 词频 | 解析快照文件（Tab 分隔），分页加载，大词典需性能优化 |
| 快照 | 调用 `rime_dict_manager`，执行前需释放词典锁 |
| 同步 | 读写 `installation.yaml`，调用同步命令，依赖 Dropbox/OneDrive 等客户端传输 |
| 跨平台 | `rime_dict_manager` 路径不同（macOS: `/usr/local/bin/`, Windows: `Rime/`），需平台适配 |
| 文件锁定 | 词典文件被输入法独占锁定，部分操作需关闭输入法 |

---

## 九、跨平台路径对照

| 操作 | macOS | Windows |
|------|-------|---------|
| 用户目录 | `~/Library/Rime` | `%AppData%\Rime` |
| 配置文件 | `squirrel.custom.yaml` | `weasel.custom.yaml` |
| 词典目录 | `luna_pinyin.userdb/` | `luna_pinyin.userdb/` |
| 词典管理工具 | `/usr/local/bin/rime_dict_manager` | `Rime/rime_dict_manager.exe` |
| 同步配置 | `installation.yaml` | `installation.yaml` |
| 同步目录 | 用户自定义（如 Dropbox 路径） | 用户自定义 |
