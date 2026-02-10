use std::{
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    ffi::{CStr, c_char, c_void},
};

use crate::{
    error::{self, builder},
    subversion,
};

use derive_new::new;
use resvg::usvg::Transform;
use snafu::ResultExt;
use tiny_skia::Pixmap;

#[easy_ext::ext(SubversionStringer)]
pub impl *const subversion::ffi::svn_string_t {
    unsafe fn to_str<'a>(self) -> &'a str {
        unsafe {
            let slice = self.to_slice();

            std::str::from_utf8(slice).unwrap()
        }
    }
    unsafe fn to_nullable_str<'a>(self) -> Option<&'a str> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_str()) }
        }
    }

    unsafe fn to_slice<'a>(self) -> &'a [u8] {
        assert!(!self.is_null());

        unsafe {
            let ptr = self.as_ref().unwrap();

            assert!(!ptr.data.is_null());

            let slice =
                std::slice::from_raw_parts(ptr.data as *const u8, ptr.len.try_into().unwrap());

            slice
        }
    }

    unsafe fn to_nullable_slice<'a>(self) -> Option<&'a [u8]> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_slice()) }
        }
    }
}

#[easy_ext::ext(CStringer)]
pub impl *const c_char {
    unsafe fn to_str<'a>(self) -> &'a str {
        assert!(!self.is_null());
        unsafe { CStr::from_ptr(self).to_str().unwrap() }
    }
    unsafe fn to_nullable_str<'a>(self) -> Option<&'a str> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_str()) }
        }
    }

    unsafe fn to_slice<'a>(self) -> &'a [u8] {
        assert!(!self.is_null());

        unsafe { CStr::from_ptr(self).to_bytes() }
    }

    unsafe fn to_nullable_slice<'a>(self) -> Option<&'a [u8]> {
        if self.is_null() {
            None
        } else {
            unsafe { Some(self.to_slice()) }
        }
    }
}

#[easy_ext::ext(Boxed)]
pub impl<T> Box<T> {
    fn inner_void_pointer(&self) -> *const c_void {
        let inner: &T = &**self;
        inner as *const T as *const c_void
    }

    fn inner_void_pointer_mut(&mut self) -> *mut c_void {
        let inner: &mut T = &mut **self;
        inner as *mut T as *mut c_void
    }
}

#[easy_ext::ext(Pointer)]
pub impl<T> T {
    fn pointer_mut(&mut self) -> *mut T {
        self as *mut _
    }

    fn pointer(&self) -> *const T {
        self as *const _
    }
}

fn rescale(
    source_width: f32,
    source_height: f32,
    target_width: f32,
    target_height: f32,
) -> Transform {
    let scale_x = target_width / source_width as f32;
    let scale_y = target_height / source_width as f32;
    let scale = scale_x.min(scale_y); // 取最小值，确保不溢出

    // 计算居中偏移（从左上角平移到中心）
    let scaled_width = source_width * scale;
    let scaled_height = source_height * scale;
    let dx = (target_width - scaled_width) / 2.0; // X偏移
    let dy = (target_height - scaled_height) / 2.0; // Y偏移

    // 构建变换：先缩放SVG，然后平移到居中位置
    let transform = Transform::from_scale(scale, scale).post_translate(dx, dy);

    transform
}

thread_local! {
    static SVG: RefCell<HashMap<String, DecodedSvg>> = Default::default();
}

#[derive(new)]
pub struct DecodedSvg {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

#[uniffi::export]
fn render_svg_to_rgba(svg_data: String, width: u32, height: u32) -> error::Result<Vec<u8>> {
    use resvg::*;
    use usvg::*;

    let data = SVG.with_borrow_mut(|svg| {
        let entry = svg.entry(svg_data.clone());

        match entry {
            Entry::Occupied(occupied_entry) => {
                if occupied_entry.get().width == width && occupied_entry.get().height == height {
                    return Some(occupied_entry.get().rgba.clone());
                }
            }
            Entry::Vacant(_) => {}
        }
        None
    });
    if let Some(data) = data {
        println!("Cache hit for SVG rendering.");
        return Ok(data);
    }

    println!("size: width={}, height={}", width, height);
    println!("Render Svg:\n{}", svg_data);

    let mut options = Options::default();
    options.fontdb_mut().load_system_fonts();

    let tree = Tree::from_str(&svg_data, &options).context(builder::Svg)?;

    let mut pixmap = Pixmap::new(width, height).unwrap();

    let transform = rescale(
        tree.size().width(),
        tree.size().height(),
        width as f32,
        height as f32,
    );
    render(&tree, transform, &mut pixmap.as_mut());

    // pixmap.save_png("./test.png").unwrap()

    let data = pixmap.take();

    SVG.with_borrow_mut(|svg| {
        svg.insert(svg_data, DecodedSvg::new(width, height, data.clone()));
    });

    Ok(data)
}
