use crate::cell::Cell;
use crate::grid::Grid;
use image::{DynamicImage, GrayImage, ImageBuffer, Luma};

/// 画像変換パラメータ。
#[derive(Debug, Clone)]
pub struct ImageConvertParams {
    /// 出力グリッドの幅（5–50）
    pub grid_width: u32,
    /// 出力グリッドの高さ（5–50）
    pub grid_height: u32,
    /// ガウシアンブラーの sigma（0 でスキップ）
    pub smooth_strength: f32,
    /// 閾値（0–255）。平均輝度 < threshold で塗りつぶし
    pub threshold: u8,
    /// エッジマージ強度（0–1）
    pub edge_strength: f32,
    /// ノイズ除去の最小連結領域サイズ（0 で無効）
    pub noise_removal: u32,
}

/// 画像変換エラー。
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    /// 画像デコード失敗。
    #[error("image decode failed: {0}")]
    Decode(#[from] image::ImageError),
}

/// 画像バイト列（PNG/JPEG/WebP/GIF）を二値 Grid に変換する。
///
/// # Errors
/// - `ImageError::Decode` — 画像デコード失敗
pub fn image_to_grid(image_bytes: &[u8], params: &ImageConvertParams) -> Result<Grid, ImageError> {
    // Step 1: デコード
    let img = image::load_from_memory(image_bytes)?;

    // Step 2: アルファチャンネルを白背景に合成
    let gray = alpha_composite_to_gray(&img);

    // Step 3 は alpha_composite_to_gray 内で実施（RGBA -> グレースケール）

    // Step 4: ガウシアンブラー（smooth_strength = 0 のときスキップ）
    let blur_buf: GrayImage;
    let blurred: &GrayImage = if params.smooth_strength > 0.0 {
        blur_buf = imageproc::filter::gaussian_blur_f32(&gray, params.smooth_strength);
        &blur_buf
    } else {
        &gray
    };

    // Step 5: Canny エッジ検出
    let edges = imageproc::edges::canny(
        blurred,
        params.threshold as f32 * 0.5,
        params.threshold as f32,
    );

    // Step 6: エッジマージ（merged = gray * (1 - edge_strength) + edge * edge_strength）
    let merged = merge_edge(blurred, &edges, params.edge_strength);

    // Step 7: セル平均ダウンサンプリング
    let grid_w = params.grid_width as usize;
    let grid_h = params.grid_height as usize;
    let cell_averages = downsample(&merged, grid_w, grid_h);

    // Step 8: 閾値処理（平均輝度 < threshold → true）
    let threshold = params.threshold as f32;
    let mut cells: Vec<Vec<bool>> = cell_averages
        .iter()
        .map(|row| row.iter().map(|&v| v < threshold).collect())
        .collect();

    // Step 9: ノイズ除去
    if params.noise_removal > 0 {
        cells = remove_noise(cells, grid_w, grid_h, params.noise_removal as usize);
    }

    // Grid に変換
    let mut grid = Grid::new(grid_h, grid_w);
    for (r, row) in cells.iter().enumerate() {
        for (c, &filled) in row.iter().enumerate() {
            grid.set(r, c, Cell::from(filled));
        }
    }

    Ok(grid)
}

/// RGBA 画像を白背景にアルファ合成してグレースケールに変換する。
fn alpha_composite_to_gray(img: &DynamicImage) -> GrayImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut gray = GrayImage::new(w, h);
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = a as f32 / 255.0;
        let cr = r as f32 * alpha + 255.0 * (1.0 - alpha);
        let cg = g as f32 * alpha + 255.0 * (1.0 - alpha);
        let cb = b as f32 * alpha + 255.0 * (1.0 - alpha);
        let luma = (0.299 * cr + 0.587 * cg + 0.114 * cb) as u8;
        gray.put_pixel(x, y, Luma([luma]));
    }
    gray
}

/// グレー画像とエッジ画像をマージする。
fn merge_edge(gray: &GrayImage, edges: &GrayImage, edge_strength: f32) -> GrayImage {
    let (w, h) = gray.dimensions();
    let mut merged: ImageBuffer<Luma<u8>, Vec<u8>> = GrayImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let gv = gray.get_pixel(x, y).0[0] as f32;
            let ev = edges.get_pixel(x, y).0[0] as f32;
            let mv = (gv * (1.0 - edge_strength) + ev * edge_strength).clamp(0.0, 255.0) as u8;
            merged.put_pixel(x, y, Luma([mv]));
        }
    }
    merged
}

/// グリッドセル (row, col) の平均輝度を計算する。
fn cell_average(img: &GrayImage, row: usize, col: usize, grid_w: usize, grid_h: usize) -> f32 {
    let (iw, ih) = img.dimensions();
    let x0 = (col as f32 * iw as f32 / grid_w as f32) as u32;
    let x1 = (((col + 1) as f32 * iw as f32 / grid_w as f32) as u32).min(iw);
    let y0 = (row as f32 * ih as f32 / grid_h as f32) as u32;
    let y1 = (((row + 1) as f32 * ih as f32 / grid_h as f32) as u32).min(ih);
    let mut sum = 0u64;
    let mut count = 0u64;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += img.get_pixel(x, y).0[0] as u64;
            count += 1;
        }
    }
    if count > 0 { sum as f32 / count as f32 } else { 255.0 }
}

/// グリッドセルごとの平均輝度を計算する。
fn downsample(img: &GrayImage, grid_w: usize, grid_h: usize) -> Vec<Vec<f32>> {
    (0..grid_h)
        .map(|row| (0..grid_w).map(|col| cell_average(img, row, col, grid_w, grid_h)).collect())
        .collect()
}

/// 単一連結成分を DFS で収集し、ラベルを付与して返す。
fn flood_fill(
    cells: &[Vec<bool>],
    labels: &mut [Vec<usize>],
    start: (usize, usize),
    label: usize,
    height: usize,
    width: usize,
) -> Vec<(usize, usize)> {
    let mut stack = vec![start];
    let mut component = Vec::new();
    labels[start.0][start.1] = label;
    while let Some((r, c)) = stack.pop() {
        component.push((r, c));
        for (nr, nc) in neighbors_4(r, c, height, width) {
            if cells[nr][nc] && labels[nr][nc] == 0 {
                labels[nr][nc] = label;
                stack.push((nr, nc));
            }
        }
    }
    component
}

/// 4-connectivity の連結成分でサイズ < min_size の成分を除去する。
fn remove_noise(mut cells: Vec<Vec<bool>>, width: usize, height: usize, min_size: usize) -> Vec<Vec<bool>> {
    let mut labels = vec![vec![0usize; width]; height];
    let mut label = 0usize;
    let mut components: Vec<Vec<(usize, usize)>> = Vec::new();

    for row in 0..height {
        for col in 0..width {
            if cells[row][col] && labels[row][col] == 0 {
                label += 1;
                components.push(flood_fill(&cells, &mut labels, (row, col), label, height, width));
            }
        }
    }

    for component in &components {
        if component.len() < min_size {
            for &(r, c) in component {
                cells[r][c] = false;
            }
        }
    }
    cells
}

fn neighbors_4(r: usize, c: usize, height: usize, width: usize) -> impl Iterator<Item = (usize, usize)> {
    let mut arr = [(0usize, 0usize); 4];
    let mut n = 0;
    if r > 0 { arr[n] = (r - 1, c); n += 1; }
    if r + 1 < height { arr[n] = (r + 1, c); n += 1; }
    if c > 0 { arr[n] = (r, c - 1); n += 1; }
    if c + 1 < width { arr[n] = (r, c + 1); n += 1; }
    arr.into_iter().take(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;

    /// テスト用白黒 PNG を生成する。
    /// top_left_black: true の場合、左上 (width/2 x height/2) を黒、残りを白にする。
    fn make_test_png(width: u32, height: u32, black_region: &[(u32, u32)]) -> Vec<u8> {
        let mut img = GrayImage::from_pixel(width, height, Luma([255u8]));
        for &(x, y) in black_region {
            if x < width && y < height {
                img.put_pixel(x, y, Luma([0u8]));
            }
        }
        let dyn_img = DynamicImage::ImageLuma8(img);
        let mut bytes = Vec::new();
        dyn_img
            .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("PNG 生成失敗");
        bytes
    }

    fn default_params(grid_width: u32, grid_height: u32) -> ImageConvertParams {
        ImageConvertParams {
            grid_width,
            grid_height,
            smooth_strength: 0.0,
            threshold: 128,
            edge_strength: 0.0,
            noise_removal: 0,
        }
    }

    #[test]
    fn known_image_produces_expected_grid() {
        // 4x4 画像: 左上 2x2 が黒、残りが白
        // grid 2x2 で変換 → cells[0][0] = true, 他 = false
        let black_pixels: Vec<(u32, u32)> = (0..2).flat_map(|y| (0..2).map(move |x| (x, y))).collect();
        let png = make_test_png(4, 4, &black_pixels);
        let params = default_params(2, 2);
        let grid = image_to_grid(&png, &params).expect("変換失敗");
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.width(), 2);
        assert_eq!(grid.get(0, 0), Cell::Filled);
        assert_eq!(grid.get(0, 1), Cell::Blank);
        assert_eq!(grid.get(1, 0), Cell::Blank);
        assert_eq!(grid.get(1, 1), Cell::Blank);
    }

    #[test]
    fn noise_removal_zero_keeps_isolated_pixel() {
        // 10x10 画像: 1 ピクセルだけ黒
        let png = make_test_png(10, 10, &[(5, 5)]);
        let mut params = default_params(10, 10);
        params.noise_removal = 0;
        let grid = image_to_grid(&png, &params).expect("変換失敗");
        // 孤立ピクセルが残っているか確認
        let filled_count = (0..10)
            .flat_map(|r| (0..10).map(move |c| (r, c)))
            .filter(|&(r, c)| grid.get(r, c) == Cell::Filled)
            .count();
        assert_eq!(filled_count, 1);
    }

    #[test]
    fn noise_removal_positive_removes_isolated_pixel() {
        // 10x10 画像: 1 ピクセルだけ黒 → noise_removal=2 で除去される
        let png = make_test_png(10, 10, &[(5, 5)]);
        let mut params = default_params(10, 10);
        params.noise_removal = 2;
        let grid = image_to_grid(&png, &params).expect("変換失敗");
        let filled_count = (0..10)
            .flat_map(|r| (0..10).map(move |c| (r, c)))
            .filter(|&(r, c)| grid.get(r, c) == Cell::Filled)
            .count();
        assert_eq!(filled_count, 0);
    }

    #[test]
    fn invalid_bytes_returns_decode_error() {
        let bad_bytes = b"this is not an image";
        let params = default_params(10, 10);
        let err = image_to_grid(bad_bytes, &params).unwrap_err();
        assert!(matches!(err, ImageError::Decode(_)));
    }
}
