//! 主题外观页（对应 ThemeEditor.vue）。
//!
//! 三 Tab：亮色主题 / 暗色主题 / 样式。
//! 主题列表（色块预览 + 应用/编辑/复制/删除）；配色编辑模态（7 色 ColorPicker）；
//! 候选窗实时预览；样式表单（字体/窗口/尺寸/行为）保存 save_style。
use std::collections::HashMap;

use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants as _},
    Disableable as _,
    scroll::ScrollableElement as _,
    checkbox::Checkbox,
    color_picker::{ColorPicker, ColorPickerState},
    dialog::DialogButtonProps,
    form::{Field, Form},
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    IndexPath,
    WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::StatefulInteractiveElement as _;
use gpui_kit::*;
use rime_core::rime::color::RimeColor;
use rime_core::rime::color_scheme::RimeColorScheme;
use rime_core::rime::style::RimeStyle;
use rime_core::service::settings::PendingDelete;
use rime_core::service::style::{self, StyleData};

fn color_to_hsla(c: &Option<RimeColor>) -> Hsla {
    match c {
        Some(c) => {
            if c.a >= 255 {
                rgb(((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32).into()
            } else {
                // c.r/g/b/a 已由 RimeColor::from_hex 解码为真实通道值；
                // gpui rgba() 打包顺序为 0xRRGGBBAA（M2：此前按 AABBGGRR 原样重排，四通道全错位）
                rgba(
                    ((c.r as u32) << 24) | ((c.g as u32) << 16) | ((c.b as u32) << 8) | c.a as u32,
                )
                .into()
            }
        }
        None => rgb(0x888888).into(),
    }
}

/// 候选窗预览（对应 Vue 版实时预览：2 行假候选）。
fn candidate_preview(_cx: &App, scheme: &RimeColorScheme) -> AnyElement {
    let back = color_to_hsla(&scheme.back_color);
    let text = color_to_hsla(&scheme.text_color);
    let cand_text = color_to_hsla(&scheme.candidate_text_color);
    let hilited_cand_back = color_to_hsla(&scheme.hilited_candidate_back_color);
    let hilited_cand_text = color_to_hsla(&scheme.hilited_candidate_text_color);
    let hilited_label = color_to_hsla(&scheme.hilited_candidate_label_color);
    let label = color_to_hsla(&scheme.candidate_label_color);

    let candidate_row = |idx: &'static str, mark: bool| {
        h_flex()
            .gap_3()
            .px_3()
            .py_1p5()
            .rounded_md()
            .when(mark, |el| el.bg(hilited_cand_back))
            .child(
                div()
                    .text_sm()
                    .text_color(if mark { hilited_label } else { label })
                    .child(idx),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(if mark { hilited_cand_text } else { cand_text })
                    .child(if mark { "nihao" } else { "zhongwen" }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(text)
                    .child(if mark { "你好" } else { "中文" }),
            )
    };

    v_flex()
        .w(px(360.))
        .rounded_lg()
        .border_1()
        .border_color(back)
        .bg(back)
        .p_2()
        .gap_1()
        .child(div().text_xs().text_color(text).child("ni hao shi jie"))
        .child(candidate_row("1.", true))
        .child(candidate_row("2.", false))
        .child(candidate_row("3.", false))
        .into_any_element()
}

type StringSelect = Entity<SelectState<Vec<String>>>;

/// 字号预设（当前值不在列表时会自动并入）。
const FONT_POINT_PRESETS: &[f64] = &[10., 11., 12., 13., 14., 15., 16., 17., 18., 20., 22., 24., 28., 32.];

fn point_options(current: f64) -> Vec<String> {
    let mut vals: Vec<f64> = FONT_POINT_PRESETS.to_vec();
    if !vals.contains(&current) {
        vals.push(current);
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    }
    vals.iter().map(|v| {
        if v.fract() == 0.0 { format!("{}", *v as i64) } else { format!("{}", v) }
    }).collect()
}

fn fmt_f64(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

fn nearest_index(options: &[String], current: f64) -> Option<IndexPath> {
    options
        .iter()
        .position(|o| o.parse::<f64>().map(|v| (v - current).abs() < 0.001).unwrap_or(false))
        .map(|row| IndexPath::default().row(row))
}

/// 颜色编辑项（key, 标签）。
const COLOR_FIELDS: &[(&str, &str)] = &[
    ("back_color", "背景色"),
    ("text_color", "文字色"),
    ("candidate_text_color", "候选文字"),
    ("hilited_back_color", "高亮背景"),
    ("hilited_candidate_text_color", "高亮文字"),
    ("candidate_label_color", "编号色"),
    ("hilited_candidate_text_color2", "高亮编号色"),
];

pub struct ThemePage {
    data: StyleData,
    schemes_cache: Vec<(String, RimeColorScheme)>, // 名称 → 方案（亮+暗合并键）
    active_tab: usize,                             // 0 亮 1 暗 2 样式
    selected_scheme: Option<String>,
    /// 已点「应用主题」但尚未部署生效的选择。
    pending_light: Option<String>,
    pending_dark: Option<String>,
    /// 待删除主题名。
    pending: Option<String>,
    // 样式 tab 的受控字段（完整保存）
    style: RimeStyle,
    // 样式表单控件（load 后创建；订阅写回 self.style）
    font_face_select: Option<StringSelect>,
    label_font_select: Option<StringSelect>,
    comment_font_select: Option<StringSelect>,
    font_point_select: Option<StringSelect>,
    label_point_select: Option<StringSelect>,
    comment_point_select: Option<StringSelect>,
    layout_select: Option<StringSelect>,
    orientation_select: Option<StringSelect>,
    status_select: Option<StringSelect>,
    format_input: Option<Entity<InputState>>,
    corner_radius_input: Option<Entity<InputState>>,
    hilited_radius_input: Option<Entity<InputState>>,
    line_spacing_input: Option<Entity<InputState>>,
    spacing_input: Option<Entity<InputState>>,
    border_height_input: Option<Entity<InputState>>,
    border_width_input: Option<Entity<InputState>>,
    shadow_size_input: Option<Entity<InputState>>,
    alpha_input: Option<Entity<InputState>>,
    _style_subs: Vec<Subscription>,
    // 编辑配色模态
    edit_color_pickers: Option<Vec<Entity<ColorPickerState>>>,
    edit_name: Option<Entity<InputState>>,
    edit_author: Option<Entity<InputState>>,
    edit_original: Option<String>, // Some=编辑原名；None=新建复制
}

impl ThemePage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            data: StyleData {
                style: RimeStyle::default(),
                light_schemes: HashMap::new(),
                dark_schemes: HashMap::new(),
                selected_light: String::new(),
                selected_dark: String::new(),
            },
            schemes_cache: Vec::new(),
            active_tab: 0,
            selected_scheme: None,
            pending_light: None,
            pending_dark: None,
            pending: None,
            style: RimeStyle::default(),
            edit_color_pickers: None,
            edit_name: None,
            edit_author: None,
            edit_original: None,
            font_face_select: None,
            label_font_select: None,
            comment_font_select: None,
            font_point_select: None,
            label_point_select: None,
            comment_point_select: None,
            layout_select: None,
            orientation_select: None,
            status_select: None,
            format_input: None,
            corner_radius_input: None,
            hilited_radius_input: None,
            line_spacing_input: None,
            spacing_input: None,
            border_height_input: None,
            border_width_input: None,
            shadow_size_input: None,
            alpha_input: None,
            _style_subs: Vec::new(),
        };
        this.load(window, cx);
        this.create_style_controls(window, cx);
        this
    }

    /// 创建样式表单的全部控件（按当前 style 值初始化）。
    fn create_style_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut families: Vec<String> = cx.text_system().all_font_names();
        families.sort();
        families.dedup();
        let s = self.style.clone();
        let mk_font = |current: &str, window: &mut Window, cx: &mut Context<Self>| -> StringSelect {
            let mut opts = families.clone();
            if !opts.iter().any(|o| o == current) {
                opts.insert(0, current.to_string());
            }
            let default = opts
                .iter()
                .position(|o| o == current)
                .map(|row| IndexPath::default().row(row));
            cx.new(|cx| SelectState::new(opts, default, window, cx).searchable(true))
        };
        self.font_face_select = Some(mk_font(&s.font_face, window, cx));
        self.label_font_select = Some(mk_font(&s.label_font_face, window, cx));
        self.comment_font_select = Some(mk_font(&s.comment_font_face, window, cx));

        let mk_point = |current: f64, window: &mut Window, cx: &mut Context<Self>| -> StringSelect {
            let opts = point_options(current);
            let default = nearest_index(&opts, current);
            cx.new(|cx| SelectState::new(opts, default, window, cx))
        };
        self.font_point_select = Some(mk_point(s.font_point, window, cx));
        self.label_point_select = Some(mk_point(s.label_font_point, window, cx));
        self.comment_point_select = Some(mk_point(s.comment_font_point, window, cx));

        let mk_choice = |options: &[&str], current: &str, window: &mut Window, cx: &mut Context<Self>| -> StringSelect {
            let labels: Vec<String> = options
                .iter()
                .map(|o| o.to_string())
                .collect();
            let default = options
                .iter()
                .position(|o| *o == current)
                .map(|row| IndexPath::default().row(row));
            cx.new(|cx| SelectState::new(labels, default, window, cx))
        };
        self.layout_select = Some(mk_choice(&["stacked", "linear"], &s.candidate_list_layout, window, cx));
        self.orientation_select = Some(mk_choice(&["horizontal", "vertical"], &s.text_orientation, window, cx));
        self.status_select = Some(mk_choice(&["mix", "long", "short", "never"], &s.status_message_type, window, cx));

        let mk_text = |v: String, window: &mut Window, cx: &mut Context<Self>| -> Entity<InputState> {
            cx.new(|cx| InputState::new(window, cx).default_value(v))
        };
        self.format_input = Some(mk_text(s.candidate_format.clone(), window, cx));
        self.corner_radius_input = Some(mk_text(fmt_f64(s.corner_radius), window, cx));
        self.hilited_radius_input = Some(mk_text(fmt_f64(s.hilited_corner_radius), window, cx));
        self.line_spacing_input = Some(mk_text(fmt_f64(s.line_spacing), window, cx));
        self.spacing_input = Some(mk_text(fmt_f64(s.spacing), window, cx));
        self.border_height_input = Some(mk_text(fmt_f64(s.border_height), window, cx));
        self.border_width_input = Some(mk_text(fmt_f64(s.border_width), window, cx));
        self.shadow_size_input = Some(mk_text(fmt_f64(s.shadow_size), window, cx));
        self.alpha_input = Some(mk_text(fmt_f64(s.alpha), window, cx));

        // 订阅写回 self.style
        let subs = &mut self._style_subs;
        subs.clear();
        macro_rules! sub_select {
            ($sel:expr, $field:ident) => {
                subs.push(cx.subscribe_in(&$sel.clone(), window, |this, _, ev: &SelectEvent<_>, _, cx| {
                    if let SelectEvent::Confirm(Some(label)) = ev {
                        this.style.$field = label.to_string();
                        cx.notify();
                    }
                }));
            };
        }
        sub_select!(self.font_face_select.as_ref().unwrap(), font_face);
        sub_select!(self.label_font_select.as_ref().unwrap(), label_font_face);
        sub_select!(self.comment_font_select.as_ref().unwrap(), comment_font_face);
        // 字号：label 解析为 f64 写回
        macro_rules! sub_point {
            ($sel:expr, $field:ident) => {
                subs.push(cx.subscribe_in(&$sel.clone(), window, |this, _, ev: &SelectEvent<_>, _, cx| {
                    if let SelectEvent::Confirm(Some(label)) = ev {
                        if let Ok(v) = label.parse::<f64>() {
                            this.style.$field = v;
                        }
                        cx.notify();
                    }
                }));
            };
        }
        sub_point!(self.font_point_select.as_ref().unwrap(), font_point);
        sub_point!(self.label_point_select.as_ref().unwrap(), label_font_point);
        sub_point!(self.comment_point_select.as_ref().unwrap(), comment_font_point);
        sub_select!(self.layout_select.as_ref().unwrap(), candidate_list_layout);
        sub_select!(self.orientation_select.as_ref().unwrap(), text_orientation);
        sub_select!(self.status_select.as_ref().unwrap(), status_message_type);
        // 数值/文本输入
        macro_rules! sub_text_num {
            ($input:expr, $field:ident) => {
                subs.push(cx.subscribe_in(&$input.clone(), window, |this, input, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        if let Ok(v) = input.read(cx).value().trim().parse::<f64>() {
                            this.style.$field = v;
                            cx.notify();
                        }
                    }
                }));
            };
        }
        sub_text_num!(self.corner_radius_input.as_ref().unwrap(), corner_radius);
        sub_text_num!(self.hilited_radius_input.as_ref().unwrap(), hilited_corner_radius);
        sub_text_num!(self.line_spacing_input.as_ref().unwrap(), line_spacing);
        sub_text_num!(self.spacing_input.as_ref().unwrap(), spacing);
        sub_text_num!(self.border_height_input.as_ref().unwrap(), border_height);
        sub_text_num!(self.border_width_input.as_ref().unwrap(), border_width);
        sub_text_num!(self.shadow_size_input.as_ref().unwrap(), shadow_size);
        sub_text_num!(self.alpha_input.as_ref().unwrap(), alpha);
        // 候选格式（文本）
        subs.push(cx.subscribe_in(&self.format_input.clone().unwrap(), window,
            |this, input, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.style.candidate_format = input.read(cx).value().to_string();
                    cx.notify();
                }
            }));
    }

    /// 数据重载后把控件值刷成 style 当前值（set_value 不触发 Change，无循环）。
    fn sync_style_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let s = self.style.clone();
        let set_sel = |sel: &Option<StringSelect>, value: &str, window: &mut Window, cx: &mut Context<Self>| {
            if let Some(sel) = sel {
                let value = value.to_string();
                sel.update(cx, |st, cx| {
                    st.set_selected_value(&value, window, cx);
                });
            }
        };
        set_sel(&self.font_face_select, &s.font_face, window, cx);
        set_sel(&self.label_font_select, &s.label_font_face, window, cx);
        set_sel(&self.comment_font_select, &s.comment_font_face, window, cx);
        set_sel(&self.font_point_select, &fmt_f64(s.font_point), window, cx);
        set_sel(&self.label_point_select, &fmt_f64(s.label_font_point), window, cx);
        set_sel(&self.comment_point_select, &fmt_f64(s.comment_font_point), window, cx);
        set_sel(&self.layout_select, &s.candidate_list_layout, window, cx);
        set_sel(&self.orientation_select, &s.text_orientation, window, cx);
        set_sel(&self.status_select, &s.status_message_type, window, cx);
        let set_text = |e: &Option<Entity<InputState>>, v: String, window: &mut Window, cx: &mut Context<Self>| {
            if let Some(e) = e {
                e.update(cx, |st, cx| st.set_value(v.as_str(), window, cx));
            }
        };
        set_text(&self.format_input, s.candidate_format.clone(), window, cx);
        set_text(&self.corner_radius_input, fmt_f64(s.corner_radius), window, cx);
        set_text(&self.hilited_radius_input, fmt_f64(s.hilited_corner_radius), window, cx);
        set_text(&self.line_spacing_input, fmt_f64(s.line_spacing), window, cx);
        set_text(&self.spacing_input, fmt_f64(s.spacing), window, cx);
        set_text(&self.border_height_input, fmt_f64(s.border_height), window, cx);
        set_text(&self.border_width_input, fmt_f64(s.border_width), window, cx);
        set_text(&self.shadow_size_input, fmt_f64(s.shadow_size), window, cx);
        set_text(&self.alpha_input, fmt_f64(s.alpha), window, cx);
        cx.notify();
    }

    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match style::get_style_data() {
            Ok(data) => {
                self.style = data.style.clone();
                self.selected_scheme = Some(data.selected_light.clone());
                // 合并亮/暗主题名（亮优先）
                let mut names: Vec<String> = data.light_schemes.keys().cloned().collect();
                for k in data.dark_schemes.keys() {
                    if !names.contains(k) {
                        names.push(k.clone());
                    }
                }
                names.sort();
                self.schemes_cache = names
                    .into_iter()
                    .map(|name| {
                        let scheme = data
                            .light_schemes
                            .get(&name)
                            .or_else(|| data.dark_schemes.get(&name))
                            .cloned()
                            .unwrap_or_default();
                        (name, scheme)
                    })
                    .collect();
                self.pending_light = None;
                self.pending_dark = None;
                self.data = data;
            }
            Err(e) => eprintln!("Warning: load style data failed: {}", e),
        }
        if self.font_face_select.is_some() {
            self.sync_style_controls(window, cx);
        }
        cx.notify();
        let _ = window;
    }

    fn current_schemes(&self) -> &HashMap<String, RimeColorScheme> {
        if self.active_tab == 1 {
            &self.data.dark_schemes
        } else {
            &self.data.light_schemes
        }
    }

    pub fn pending_deletes(&self) -> Vec<PendingDelete> {
        self.pending
            .iter()
            .map(|f| PendingDelete {
                delete_type: "theme".to_string(),
                identifier: f.clone(),
            })
            .collect()
    }

    pub fn on_deploy_complete(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.pending = None;
        // 重载 StyleData + 清 pending_light/pending_dark（load 内已做，M4：此前内存态过期）
        self.load(window, cx);
    }

    /// 应用主题：保存选择（亮/暗），显示「待生效」直至部署完成。
    fn apply_scheme(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let light = if self.active_tab == 0 {
            name.clone()
        } else {
            self.data.selected_light.clone()
        };
        let dark = if self.active_tab == 1 {
            name.clone()
        } else {
            self.data.selected_dark.clone()
        };
        if let Err(e) = style::save_selected_schemes(light.clone(), dark.clone()) {
            window.push_notification(format!("应用主题失败：{}", e), cx);
            return;
        }
        if self.active_tab == 0 {
            self.pending_light = Some(name.clone());
        } else {
            self.pending_dark = Some(name.clone());
        }
        cx.notify();
    }

    fn cancel_apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.active_tab == 0 {
            self.pending_light = None;
        } else {
            self.pending_dark = None;
        }
        self.load(window, cx);
    }

    fn save_style(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = style::save_style(self.style.clone()) {
            window.push_notification(format!("保存样式失败：{}", e), cx);
            return;
        }
        window.push_notification("样式已保存", cx);
    }

    fn import_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // rfd 同步对话框会在 GPUI 事件回调内运行模态循环，重入 App 借用会 panic；
        // 改为后台线程异步选择，回调回 UI 线程。
        cx.spawn_in(window, async move |this, cx| {
            let picked = cx
                .background_spawn(async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("配色方案 (.yaml)", &["yaml", "yml"])
                        .set_title("选择配色方案文件")
                        .pick_file()
                        .await
                })
                .await;
            let Some(file) = picked else { return };
            let path = file.path().display().to_string();
            this.update_in(cx, |this, window, cx| {
                if let Err(e) = style::import_color_scheme(path) {
                    window.push_notification(format!("导入主题失败：{}", e), cx);
                    return;
                }
                this.load(window, cx);
                window.push_notification("主题已导入", cx);
            })
            .ok();
        })
        .detach();
    }

    fn open_scheme_editor(&mut self, name: Option<String>, copy: bool, window: &mut Window, cx: &mut Context<Self>) {
        // name=None → 编辑当前选中；copy=true → 复制模式
        let source_name = name.or_else(|| self.selected_scheme.clone());
        let source = source_name
            .as_ref()
            .and_then(|n| self.current_schemes().get(n))
            .cloned()
            .unwrap_or_default();

        let pickers: Vec<Entity<ColorPickerState>> = COLOR_FIELDS
            .iter()
            .map(|(key, _)| {
                let color = field_color(&source, key);
                cx.new(|cx| {
                    ColorPickerState::new(window, cx).default_value(color_to_hsla(&Some(color)))
                })
            })
            .collect();
        let default_name = match (&source_name, copy) {
            (Some(n), true) => format!("{}_copy", n),
            (Some(n), _) => n.clone(),
            (None, _) => String::new(),
        };
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("方案名称")
                .default_value(default_name)
        });
        let author_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("作者")
                .default_value(source.author.clone())
        });
        self.edit_color_pickers = Some(pickers);
        self.edit_name = Some(name_input.clone());
        self.edit_author = Some(author_input.clone());
        self.edit_original = if copy { None } else { source_name.clone() };

        let weak = cx.weak_entity();
        let pickers2 = self.edit_color_pickers.clone().unwrap();
        let name_input2 = name_input.clone();
        let author_input2 = author_input.clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let weak = weak.clone();
            let pickers = pickers2.clone();
            let name_input = name_input2.clone();
            let author_input = author_input2.clone();
            dialog
                .title("编辑配色")
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            Form::horizontal()
                                .label_width(px(120.))
                                .child(Field::new().label("名称").child(Input::new(&name_input)))
                                .child(Field::new().label("作者").child(Input::new(&author_input))),
                        )
                        .child(
                            v_flex()
                                .gap_2()
                                .children(COLOR_FIELDS.iter().enumerate().map(|(i, (_, label))| {
                                    let picker = pickers.get(i).cloned().unwrap();
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .child(div().w(px(100.)).text_sm().child(*label))
                                        .child(ColorPicker::new(&picker))
                                        .into_any_element()
                                })),
                        ),
                )
                .button_props(
                    DialogButtonProps::default().ok_text("保存").on_ok({
                        let weak = weak.clone();
                        let name_input = name_input.clone();
                        let author_input = author_input.clone();
                        let pickers = pickers.clone();
                        move |_, window, cx| {
                            let _ = weak.update(cx, |this, cx| {
                                this.submit_scheme_edit(&name_input, &author_input, &pickers, window, cx);
                            });
                            true
                        }
                    }),
                )
                .overlay_closable(true)
        });
    }

    fn submit_scheme_edit(
        &mut self,
        name_input: &Entity<InputState>,
        author_input: &Entity<InputState>,
        pickers: &[Entity<ColorPickerState>],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = name_input.read(cx).value().trim().to_string();
        if name.is_empty() {
            window.push_notification("方案名称不能为空", cx);
            return;
        }
        let original = self.edit_original.clone().filter(|o| *o != name);
        let mut scheme = self
            .edit_original
            .as_ref()
            .and_then(|n| self.current_schemes().get(n))
            .cloned()
            .unwrap_or_default();
        scheme.name = name.clone();
        scheme.author = author_input.read(cx).value().to_string();
        for (i, (key, _)) in COLOR_FIELDS.iter().enumerate() {
            let Some(picker) = pickers.get(i) else { continue };
            let Some(hsla) = picker.read(cx).value() else { continue };
            let color = hsla_to_rime_color(hsla);
            set_field_color(&mut scheme, key, Some(color));
        }
        if let Err(e) = style::save_color_scheme(name.clone(), scheme, original) {
            window.push_notification(format!("保存配色失败：{}", e), cx);
            return;
        }
        self.load(window, cx);
        window.push_notification("配色已保存", cx);
    }

    fn toggle_pending(&mut self, cx: &mut Context<Self>) {
        let Some(name) = self.selected_scheme.clone() else { return };
        match &self.pending {
            Some(p) if *p == name => self.pending = None,
            _ => self.pending = Some(name),
        }
        cx.notify();
    }
}

fn field_color(scheme: &RimeColorScheme, key: &str) -> RimeColor {
    let c = match key {
        "back_color" => &scheme.back_color,
        "text_color" => &scheme.text_color,
        "candidate_text_color" => &scheme.candidate_text_color,
        "hilited_back_color" => &scheme.hilited_back_color,
        "hilited_candidate_text_color" | "hilited_candidate_text_color2" => {
            &scheme.hilited_candidate_text_color
        }
        "candidate_label_color" => &scheme.candidate_label_color,
        _ => &None,
    };
    c.clone().unwrap_or(RimeColor::new(0xff, 0xff, 0xff, 0xff))
}

fn set_field_color(scheme: &mut RimeColorScheme, key: &str, color: Option<RimeColor>) {
    match key {
        "back_color" => scheme.back_color = color,
        "text_color" => scheme.text_color = color,
        "candidate_text_color" => scheme.candidate_text_color = color,
        "hilited_back_color" => scheme.hilited_back_color = color,
        "hilited_candidate_text_color" | "hilited_candidate_text_color2" => {
            scheme.hilited_candidate_text_color = color
        }
        "candidate_label_color" => scheme.candidate_label_color = color,
        _ => {}
    }
}

fn hsla_to_rime_color(hsla: Hsla) -> RimeColor {
    let rgba: Rgba = hsla.into();
    let to_u8 = |v: f32| (v * 255.0).round().clamp(0.0, 255.0) as u8;
    RimeColor::new(to_u8(rgba.r), to_u8(rgba.g), to_u8(rgba.b), 0xff)
}

impl Render for ThemePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;
        let green = gpui_kit::rgb(0x22c55e);
        let orange = gpui_kit::rgb(0xf59e0b);
        let danger = cx_theme.danger;

        // Tab 条
        let tabs = h_flex()
            .gap_2()
            .children(["亮色主题", "暗色主题", "样式"].iter().enumerate().map(|(ix, label)| {
                let active = self.active_tab == ix;
                Button::new(SharedString::from(format!("theme-tab-{ix}")))
                    .when(active, |el| el.primary())
                    .when(!active, |el| el.ghost())
                    .label(*label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active_tab = ix;
                        cx.notify();
                    }))
            }));

        let content: AnyElement = if self.active_tab <= 1 {
            // ── 主题列表 ──
            let scheme_names: Vec<String> = {
                let schemes = self.current_schemes();
                let mut v: Vec<String> = schemes.keys().cloned().collect();
                v.sort();
                v
            };
            let selected = self.selected_scheme.clone();
            let rows: Vec<AnyElement> = scheme_names
                .iter()
                .cloned()
                .enumerate()
                .map(|(ix, name)| {
                    let scheme = self.current_schemes().get(&name).cloned().unwrap_or_default();
                    let is_selected = selected.as_deref() == Some(name.as_str());
                    let is_current = (self.active_tab == 0
                        && self.data.selected_light == name)
                        || (self.active_tab == 1 && self.data.selected_dark == name);
                    let is_pending = (self.active_tab == 0
                        && self.pending_light.as_deref() == Some(name.as_str()))
                        || (self.active_tab == 1
                            && self.pending_dark.as_deref() == Some(name.as_str()));
                    let is_pending_del = self.pending.as_deref() == Some(name.as_str());
                    let name2 = name.clone();
                    h_flex()
                        .id(SharedString::from(format!("scheme-{ix}")))
                        .gap_3()
                        .items_center()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(border)
                        .cursor_pointer()
                        .when(is_selected, |el| el.bg(cx_theme.secondary))
                        .hover(|s| s.bg(cx_theme.secondary))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_scheme = Some(name2.clone());
                            cx.notify();
                        }))
                        .child(
                            // 48×32 色块
                            div()
                                .w(px(48.))
                                .h(px(32.))
                                .rounded_md()
                                .border_1()
                                .border_color(border)
                                .bg(color_to_hsla(&scheme.back_color)),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .child(div().text_sm().child(name.clone()))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(muted)
                                        .child(SharedString::from(scheme.author.clone())),
                                ),
                        )
                        .when(is_current, |el| {
                            el.child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(green)
                                    .text_color(gpui_kit::black())
                                    .child("使用中"),
                            )
                        })
                        .when(is_pending && !is_current, |el| {
                            el.child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(orange)
                                    .text_color(gpui_kit::black())
                                    .child("待生效"),
                            )
                        })
                        .when(is_pending_del, |el| {
                            el.child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(danger)
                                    .text_color(cx_theme.danger_foreground)
                                    .child("待删除"),
                            )
                        })
                        .into_any_element()
                })
                .collect();

            // 预览与操作
            let preview_scheme = self
                .selected_scheme
                .as_ref()
                .and_then(|n| self.current_schemes().get(n))
                .cloned()
                .unwrap_or_default();
            let is_current = self
                .selected_scheme
                .as_ref()
                .map(|n| {
                    (self.active_tab == 0 && self.data.selected_light == *n)
                        || (self.active_tab == 1 && self.data.selected_dark == *n)
                })
                .unwrap_or(false);
            let has_pending = (self.active_tab == 0 && self.pending_light.is_some())
                || (self.active_tab == 1 && self.pending_dark.is_some());

            v_flex()
                .flex_1()
                .min_h_0()
                .gap_3()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("theme-import")
                                        .outline()
                                                                                .label("导入主题")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.import_theme(window, cx);
                                        })),
                                )
                                .when(has_pending, |el| {
                                    el.child(
                                        div().text_sm().text_color(orange).child("主题已保存，待部署后生效"),
                                    )
                                }),
                        )
                        .child(
                            Button::new("theme-save-style")
                                .primary()
                                .label("保存样式")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.save_style(window, cx);
                                })),
                        ),
                )
                .child(
                    // 外框固定（边框不随内容滚），内容在框内滚动
                    div()
                        .flex_1()
                        .min_h_0()
                        .rounded_lg()
                        .border_1()
                        .border_color(border)
                        .overflow_hidden()
                        .child(
                            div().h_full().overflow_y_scrollbar().child(div().children(rows)),
                        ),
                )
                .child(candidate_preview(cx, &preview_scheme))
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("theme-apply")
                                .primary()
                                .disabled(is_current && !has_pending)
                                .label(if has_pending {
                                    "取消应用"
                                } else if is_current {
                                    "当前使用中"
                                } else {
                                    "应用主题"
                                })
                                .on_click(cx.listener(|this, _, window, cx| {
                                    // M5：只看当前 Tab 的 pending，避免误触发另一 Tab 的取消
                                    let tab_pending = match this.active_tab {
                                        0 => this.pending_light.is_some(),
                                        _ => this.pending_dark.is_some(),
                                    };
                                    if tab_pending {
                                        this.cancel_apply(window, cx);
                                    } else if let Some(n) = this.selected_scheme.clone() {
                                        this.apply_scheme(n, window, cx);
                                    }
                                })),
                        )
                        .child(
                            Button::new("theme-edit")
                                .outline()
                                .label("编辑")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let name = this.selected_scheme.clone();
                                    this.open_scheme_editor(name, false, window, cx);
                                })),
                        )
                        .child(
                            Button::new("theme-copy")
                                .outline()
                                .label("复制")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let name = this.selected_scheme.clone();
                                    this.open_scheme_editor(name, true, window, cx);
                                })),
                        )
                        .child(
                            Button::new("theme-del")
                                .danger()
                                .label("删除")
                                .disabled(self.selected_scheme.as_deref() == Some(self.data.selected_light.as_str())
                                    || self.selected_scheme.as_deref() == Some(self.data.selected_dark.as_str()))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toggle_pending(cx);
                                })),
                        ),
                )
                .into_any_element()
        } else {
            // ── 样式表单（全控件可编辑）──
            let sel_view = |sel: &Option<StringSelect>, w: Pixels| -> AnyElement {
                match sel {
                    Some(s) => Select::new(s).w(w).into_any_element(),
                    None => div().text_sm().text_color(muted).child("加载中…").into_any_element(),
                }
            };
            let input_view = |input: &Option<Entity<InputState>>, w: Pixels| -> AnyElement {
                match input {
                    Some(e) => Input::new(e).w(w).into_any_element(),
                    None => div().text_sm().text_color(muted).child("加载中…").into_any_element(),
                }
            };

            let style_form = v_flex()
                .gap_4()
                .child(
                    // 字体节
                    Form::horizontal()
                        .label_width(px(120.))
                        .columns(2)
                        .child(
                            Field::new().label("主字体").description("候选窗主文字字体")
                                .child(sel_view(&self.font_face_select, px(220.))),
                        )
                        .child(
                            Field::new().label("主字号")
                                .child(sel_view(&self.font_point_select, px(110.))),
                        )
                        .child(
                            Field::new().label("序号字体").description("候选序号的字体")
                                .child(sel_view(&self.label_font_select, px(220.))),
                        )
                        .child(
                            Field::new().label("序号字号")
                                .child(sel_view(&self.label_point_select, px(110.))),
                        )
                        .child(
                            Field::new().label("注释字体").description("候选注释的字体")
                                .child(sel_view(&self.comment_font_select, px(220.))),
                        )
                        .child(
                            Field::new().label("注释字号")
                                .child(sel_view(&self.comment_point_select, px(110.))),
                        ),
                )
                .child(
                    // 窗口样式
                    Form::horizontal()
                        .label_width(px(120.))
                        .columns(2)
                        .child(
                            Field::new().label("候选排列").description("候选菜单的排列方向")
                                .child(sel_view(&self.layout_select, px(180.))),
                        )
                        .child(
                            Field::new().label("文字方向")
                                .child(sel_view(&self.orientation_select, px(180.))),
                        )
                        .child(
                            Field::new().label("候选格式").description("候选行的显示模板")
                                .child(input_view(&self.format_input, px(180.))),
                        )
                        .child(
                            Field::new().label("状态提示").description("编码状态提示的显示方式")
                                .child(sel_view(&self.status_select, px(180.))),
                        ),
                )
                .child(
                    // 尺寸与间距
                    Form::horizontal()
                        .label_width(px(150.))
                        .columns(2)
                        .child(
                            Field::new().label("圆角半径")
                                .child(input_view(&self.corner_radius_input, px(110.))),
                        )
                        .child(
                            Field::new().label("高亮圆角")
                                .child(input_view(&self.hilited_radius_input, px(110.))),
                        )
                        .child(
                            Field::new().label("行间距")
                                .child(input_view(&self.line_spacing_input, px(110.))),
                        )
                        .child(
                            Field::new().label("间距")
                                .child(input_view(&self.spacing_input, px(110.))),
                        )
                        .child(
                            Field::new().label("边框高度")
                                .child(input_view(&self.border_height_input, px(110.))),
                        )
                        .child(
                            Field::new().label("边框宽度")
                                .child(input_view(&self.border_width_input, px(110.))),
                        )
                        .child(
                            Field::new().label("阴影大小")
                                .child(input_view(&self.shadow_size_input, px(110.))),
                        )
                        .child(
                            Field::new().label("透明度 (0–1)")
                                .child(input_view(&self.alpha_input, px(110.))),
                        ),
                )
                .child(
                    // 行为选项（两列网格）
                    h_flex()
                        .flex_wrap()
                        .gap_2()
                        .max_w(px(640.))
                        .child(
                            Checkbox::new("style-inline-preedit")
                                .label("内嵌预编辑 (inline_preedit)")
                                .checked(self.style.inline_preedit)
                                .on_click(cx.listener(|this, c: &bool, _, cx| {
                                    this.style.inline_preedit = *c;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("style-inline-candidate")
                                .label("内嵌候选 (inline_candidate)")
                                .checked(self.style.inline_candidate)
                                .on_click(cx.listener(|this, c: &bool, _, cx| {
                                    this.style.inline_candidate = *c;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("style-translucency")
                                .label("半透明 (translucency)")
                                .checked(self.style.translucency)
                                .on_click(cx.listener(|this, c: &bool, _, cx| {
                                    this.style.translucency = *c;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("style-mutual-exclusive")
                                .label("颜色互斥 (mutual_exclusive)")
                                .checked(self.style.mutual_exclusive)
                                .on_click(cx.listener(|this, c: &bool, _, cx| {
                                    this.style.mutual_exclusive = *c;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("style-memorize")
                                .label("记忆宽度 (memorize_size)")
                                .checked(self.style.memorize_size)
                                .on_click(cx.listener(|this, c: &bool, _, cx| {
                                    this.style.memorize_size = *c;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Checkbox::new("style-show-paging")
                                .label("显示翻页 (show_paging)")
                                .checked(self.style.show_paging)
                                .on_click(cx.listener(|this, c: &bool, _, cx| {
                                    this.style.show_paging = *c;
                                    cx.notify();
                                })),
                        ),
                );
            v_flex()
                .flex_1()
                .min_h_0()
                .gap_3()
                .child(
                    h_flex()
                        .justify_end()
                        .child(
                            Button::new("style-save")
                                .primary()
                                .label("保存样式")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.save_style(window, cx);
                                })),
                        ),
                )
                .child(
                    div()
                        .id("style-form-scroll")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(style_form),
                )
                .into_any_element()
        };

        v_flex()
            .size_full()
            .gap_4()
            .child(deploy_notice(cx))
            .child(tabs)
            .child(content)
    }
}

use crate::widgets::deploy_notice;
