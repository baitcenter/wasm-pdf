use base64;
use wasm_bindgen::prelude::*;

mod canvas;
mod encoders;
mod font;
pub mod json;
mod models;
mod objects;
mod styles;
mod template;
mod text;
mod units;

use json::{JsContent, JsDocument, JsParamValue};
use models::{Cell, Document, Image, Paragraph, Row, Spacer, Table};
use styles::{get_color, get_table_style, get_paragraph_style};
use template::PageTemplate;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);
    #[wasm_bindgen(js_name = jsonOut)]
    pub fn json_out(data: &JsValue);
}

pub fn create(js_doc: &JsDocument) -> Result<Vec<u8>, JsValue> {
    // add document content to template and build
    let template = PageTemplate::new(
        units::A4,
        js_doc.template.top,
        js_doc.template.left,
        js_doc.template.right,
        js_doc.template.bottom,
    );
    let mut doc = Document::new(&js_doc.title);
    // parse contents of JSON Document
    for content in &js_doc.contents {
        match content.obj_type.as_str() {
            "Table" => {
                let table = get_table(&content, js_doc)?;
                doc.add(Box::new(table));
            }
            "Image" => {
                if let Some(image) = get_image(&content, &js_doc) {
                    doc.add(Box::new(image));
                }
            }
            "Paragraph" => {
                let paragraph = get_paragraph(&content);
                doc.add(Box::new(paragraph));
            }
            "Spacer" => {
                let spacer = get_spacer(&content);
                doc.add(Box::new(spacer));
            }
            _ => (),
        }
    }
    // build document -> return bytes
    template.build(&doc)
}

fn get_table(content: &JsContent, js_doc: &JsDocument) -> Result<Table, JsValue> {
    let table_style = get_table_style(content);
    let mut table = Table::new(table_style);
    if let Some(rows) = content.params.get("rows") {
        if let JsParamValue::Children(rows) = rows {
            for row in rows {
                let mut r = Row::new();
                if let Some(cells) = row.params.get("cells") {
                    if let JsParamValue::Children(cells) = cells {
                        //log(&format!("number of cells: {}", cells.len()));
                        for cell in cells {
                            let cell_span = if let Some(span) = cell.params.get("span") {
                                match span {
                                    JsParamValue::Number(i) => *i,
                                    _ => 1.0,
                                }
                            } else {
                                1.0
                            };
                            let mut c = Cell::new(cell_span);
                            if let Some(cell_contents) = cell.params.get("contents") {
                                if let JsParamValue::Children(contents) = cell_contents {
                                    for cell_content in contents {
                                        match cell_content.obj_type.as_str() {
                                            "Paragraph" => {
                                                let paragraph = get_paragraph(&cell_content);
                                                c.add(Box::new(paragraph));
                                            }
                                            "Image" => {
                                                if let Some(image) =
                                                    get_image(&cell_content, &js_doc)
                                                {
                                                    c.add(Box::new(image));
                                                }
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                            }
                            if let Some(cell_style) = cell.params.get("style") {
                                if let JsParamValue::Object(cell_style) = cell_style {
                                    if let Some(bg_color) = cell_style.get("background_color") {
                                        c.style.background_color = get_color(bg_color);
                                    }
                                }
                            }
                            r.add_cell(c);
                        }
                    }
                }
                table.add_row(r);
            }
        }
    }
    Ok(table)
}

fn get_image(content: &JsContent, js_doc: &JsDocument) -> Option<Image> {
    let fit_width = if let Some(fit_width) = content.params.get("fit_width") {
        if let JsParamValue::Boolean(fit_width) = fit_width {
            *fit_width
        } else {
            false
        }
    } else {
        false
    };
    if let Some(src) = content.params.get("src") {
        if let JsParamValue::Text(s) = src {
            if let Some(image_data_str) = js_doc.image_data.get(s) {
                let p_width = if let Some(width) = js_doc.image_widths.get(s) {
                    *width
                } else {
                    0.0
                };
                let p_height = if let Some(height) = js_doc.image_heights.get(s) {
                    *height
                } else {
                    0.0
                };
                let image_data = base64::decode(&image_data_str).unwrap();
                let image = Image::new(image_data, p_width, p_height, fit_width);
                return Some(image);
            }
        }
    }
    None
}

fn get_spacer(content: &JsContent) -> Spacer {
    let p_width = if let Some(width) = content.params.get("width") {
        match width {
            JsParamValue::Number(i) => *i,
            _ => 0.0,
        }
    } else {
        0.0
    };
    let p_height = if let Some(height) = content.params.get("height") {
        match height {
            JsParamValue::Number(i) => *i,
            _ => 0.0,
        }
    } else {
        0.0
    };
    Spacer::new(p_width, p_height)
}

fn get_paragraph(content: &JsContent) -> Paragraph {
    let p_font_name = if let Some(font_name) = content.params.get("font_name") {
        match font_name {
            JsParamValue::Text(s) => s.clone(),
            _ => String::from("Helvetica"),
        }
    } else {
        String::from("Helvetica")
    };
    let p_font_size = if let Some(font_size) = content.params.get("font_size") {
        if let JsParamValue::Number(font_size) = font_size {
            *font_size
        } else {
            12.0
        }
    } else {
        12.0
    };
    let p_style = get_paragraph_style(&content, p_font_size);
    let text_value = if let Some(text) = content.params.get("text") {
        match text {
            JsParamValue::Text(s) => s.clone(),
            _ => String::new(),
        }
    } else {
        String::new()
    };
    Paragraph::new(
        &text_value,
        &p_font_name,
        p_font_size,
        p_style
    )
}