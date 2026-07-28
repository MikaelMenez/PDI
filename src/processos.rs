use image::*;
use rand::Rng;
use rustfft::{FftPlanner, num_complex::Complex};
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32, // Matiz (Hue)
    pub s: f32, // Saturação (Saturation)
    pub v: f32, // Valor (Value)
}

impl Hsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        let v = (r + g + b) / 3_f32;
        let s = if v == 0_f32 {
            0_f32
        } else {
            1_f32
                - ((3_f32 * ([r, g, b].iter().fold(f32::INFINITY, |a, &b| a.min(b)))) / (r + g + b))
                    as f32
        };
        let s = s * 255_f32;
        let teta = (0.5 * ((r - g) + (r - b))
            / ((((r - g) * (r - g)) + ((r - b) * (g - b))).sqrt() + f32::EPSILON))
            .acos();
        let h = if b <= g {
            teta / 360_f32
        } else {
            (360_f32 - teta) / 360_f32
        };
        let h = h * 255_f32;
        Self { h, s, v }
    }
}

pub fn decomposicao_rgb(img: image::DynamicImage) -> Vec<(DynamicImage, String)> {
    let mut vec = vec![];
    let mut r: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut g: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut b: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut pseudor: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut pseudog: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut pseudob: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());

    for pixel in img.pixels() {
        let tempr = r.get_pixel_mut(pixel.0, pixel.1);
        *tempr = Rgb([pixel.2[0], pixel.2[0], pixel.2[0]]);
        let temppseudor = pseudor.get_pixel_mut(pixel.0, pixel.1);
        *temppseudor = Rgb([pixel.2[0], 0, 0]);
        let tempg = g.get_pixel_mut(pixel.0, pixel.1);
        *tempg = Rgb([pixel.2[1], pixel.2[1], pixel.2[1]]);
        let temppseudog = pseudog.get_pixel_mut(pixel.0, pixel.1);
        *temppseudog = Rgb([0, pixel.2[1], 0]);
        let tempb = b.get_pixel_mut(pixel.0, pixel.1);
        *tempb = Rgb([pixel.2[2], pixel.2[2], pixel.2[2]]);
        let temppseudob = pseudob.get_pixel_mut(pixel.0, pixel.1);
        *temppseudob = Rgb([0, 0, pixel.2[2]]);
    }
    vec.push((r.into(), "Canal_R".to_string()));
    vec.push((g.into(), "Canal_G".to_string()));
    vec.push((b.into(), "Canal_B".to_string()));
    vec.push((pseudor.into(), "Canal_R_pseudocoloracao".to_string()));
    vec.push((pseudog.into(), "Canal_G_pseudocoloracao".to_string()));
    vec.push((pseudob.into(), "Canal_B_pseudocoloracao".to_string()));
    vec
}
pub fn decomposicao_hsv(img: image::DynamicImage) -> Vec<(DynamicImage, String)> {
    let mut vec: Vec<(DynamicImage, String)> = vec![];
    let mut h: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut s: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut v: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    for pixel in img.pixels() {
        let hsv = Hsv::from_rgb(pixel.2[0] as f32, pixel.2[1] as f32, pixel.2[2] as f32);
        let tempr = h.get_pixel_mut(pixel.0, pixel.1);
        *tempr = Rgb([hsv.h as u8, hsv.h as u8, hsv.h as u8]);
        let tempg = s.get_pixel_mut(pixel.0, pixel.1);
        *tempg = Rgb([hsv.s as u8, hsv.s as u8, hsv.s as u8]);
        let tempb = v.get_pixel_mut(pixel.0, pixel.1);
        *tempb = Rgb([hsv.v as u8, hsv.v as u8, hsv.v as u8]);
    }

    vec.push((h.into(), "Canal_H".to_string()));
    vec.push((s.into(), "Canal_S".to_string()));
    vec.push((v.into(), "Canal_V".to_string()));

    vec
}

pub fn limiarizacao(img: DynamicImage, limiar: u8) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(4);

    let mut cinza_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut binaria_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut cinza_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut binaria_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for pixel in img.pixels() {
        let (x, y) = (pixel.0, pixel.1);
        let rgb = pixel.2;

        // 1. Média Simples
        let valor_base = ((rgb[0] as f32 + rgb[1] as f32 + rgb[2] as f32) / 3.0) as u8;
        let binario_base = if valor_base >= limiar { 255 } else { 0 };

        // 2. Percepção do Olho Humano (ITU-R BT.709)
        let valor_olho =
            (rgb[0] as f32 * 0.2126 + rgb[1] as f32 * 0.7152 + rgb[2] as f32 * 0.0722) as u8;
        let binario_olho = if valor_olho >= limiar { 255 } else { 0 };

        // Atribuição corrigida para cada buffer específico
        *cinza_base.get_pixel_mut(x, y) = Rgb([valor_base, valor_base, valor_base]);
        *binaria_base.get_pixel_mut(x, y) = Rgb([binario_base, binario_base, binario_base]);
        *cinza_olho.get_pixel_mut(x, y) = Rgb([valor_olho, valor_olho, valor_olho]);
        *binaria_olho.get_pixel_mut(x, y) = Rgb([binario_olho, binario_olho, binario_olho]);
    }

    vec.push((
        cinza_base.into(),
        "Imagem_Escala_De_Cinza_Simples".to_string(),
    ));
    vec.push((
        binaria_base.into(),
        "Imagem_Limiarizada_Simples".to_string(),
    ));
    vec.push((
        cinza_olho.into(),
        "Imagem_Escala_De_Cinza_Adaptada".to_string(),
    ));
    vec.push((
        binaria_olho.into(),
        "Imagem_Limiarizada_Adaptada".to_string(),
    ));

    vec
}

/// Aplica transformações logarítmicas (Ln, Log10 e Log2) para expansão de contraste em regiões escuras.
/// Fórmula: s = c * log(1 + r)
pub fn transformacao_log(img: DynamicImage, ganho: f32) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(8);

    let mut cinza_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut ln_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut log10_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut log2_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    let mut cinza_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut ln_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut log10_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut log2_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for pixel in img.pixels() {
        let (x, y) = (pixel.0, pixel.1);
        let rgb = pixel.2;

        // --- Nível Base (Média Simples) ---
        let int_base_f32 = (rgb[0] as f32 + rgb[1] as f32 + rgb[2] as f32) / 3.0;
        let int_base = int_base_f32 as u8;

        let int_ln_base = (ganho * (1.0 + int_base_f32).ln()).clamp(0.0, 255.0) as u8;
        let int_log10_base = (ganho * (1.0 + int_base_f32).log10()).clamp(0.0, 255.0) as u8;
        let int_log2_base = (ganho * (1.0 + int_base_f32).log2()).clamp(0.0, 255.0) as u8;

        // --- Nível Adaptado ao Olho Humano ---
        let int_olho_f32 = rgb[0] as f32 * 0.2126 + rgb[1] as f32 * 0.7152 + rgb[2] as f32 * 0.0722;
        let int_olho = int_olho_f32 as u8;

        let int_ln_olho = (ganho * (1.0 + int_olho_f32).ln()).clamp(0.0, 255.0) as u8;
        let int_log10_olho = (ganho * (1.0 + int_olho_f32).log10()).clamp(0.0, 255.0) as u8;
        let int_log2_olho = (ganho * (1.0 + int_olho_f32).log2()).clamp(0.0, 255.0) as u8;

        // Gravação nos Buffers
        *cinza_base.get_pixel_mut(x, y) = Rgb([int_base, int_base, int_base]);
        *ln_base.get_pixel_mut(x, y) = Rgb([int_ln_base, int_ln_base, int_ln_base]);
        *log10_base.get_pixel_mut(x, y) = Rgb([int_log10_base, int_log10_base, int_log10_base]);
        *log2_base.get_pixel_mut(x, y) = Rgb([int_log2_base, int_log2_base, int_log2_base]);

        *cinza_olho.get_pixel_mut(x, y) = Rgb([int_olho, int_olho, int_olho]);
        *ln_olho.get_pixel_mut(x, y) = Rgb([int_ln_olho, int_ln_olho, int_ln_olho]);
        *log10_olho.get_pixel_mut(x, y) = Rgb([int_log10_olho, int_log10_olho, int_log10_olho]);
        *log2_olho.get_pixel_mut(x, y) = Rgb([int_log2_olho, int_log2_olho, int_log2_olho]);
    }

    vec.push((
        cinza_base.into(),
        "Imagem_Escala_De_Cinza_Simples".to_string(),
    ));
    vec.push((ln_base.into(), "Transformacao_Ln_Simples".to_string()));
    vec.push((log10_base.into(), "Transformacao_Log10_Simples".to_string()));
    vec.push((log2_base.into(), "Transformacao_Log2_Simples".to_string()));

    vec.push((
        cinza_olho.into(),
        "Imagem_Escala_De_Cinza_Adaptada".to_string(),
    ));
    vec.push((ln_olho.into(), "Transformacao_Ln_Adaptada".to_string()));
    vec.push((
        log10_olho.into(),
        "Transformacao_Log10_Adaptada".to_string(),
    ));
    vec.push((log2_olho.into(), "Transformacao_Log2_Adaptada".to_string()));

    vec
}

pub fn salva_decomposicao_rgb(
    imgs: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    dir: String,
    name: String,
) -> Result<(), Box<dyn Error>> {
    imgs[0].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "R.png"
    ))?;
    imgs[1].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "G.png"
    ))?;
    imgs[2].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "B.png"
    ))?;
    imgs[3].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "pseudoR.png"
    ))?;
    imgs[4].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "pseudoG.png"
    ))?;
    imgs[5].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "pseudoB.png"
    ))?;
    Ok(())
}
pub fn transformacao_de_intensidade_de_potencia(
    img: DynamicImage,
    gama: f32,
    ganho: f32,
) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let transform = |x| (ganho * ((x as f32) / 255_f32).powf(gama) * 255.0).clamp(0.0, 255.0) as u8;
    for pixel in img.pixels() {
        let temp = saida.get_pixel_mut(pixel.0, pixel.1);
        *temp = Rgb([
            transform(pixel.2[0]),
            transform(pixel.2[1]),
            transform(pixel.2[2]),
        ]);
    }

    vec.push((saida.into(), "Transformada de potência gama".to_owned()));
    vec
}
fn n_neighbors(tam: u8, x: u32, y: u32) -> Vec<(i64, i64)> {
    let mut vec: Vec<(i64, i64)> = Vec::with_capacity(tam as usize * tam as usize);
    let offset = (tam / 2) as i64;

    for dy in -offset..=offset {
        for dx in -offset..=offset {
            let nx = (x as i64) + dx;
            let ny = (y as i64) + dy;
            vec.push((nx, ny));
        }
    }

    vec
}

pub fn filtro_mediana(img: DynamicImage, tam: u8, placeholder: u8) -> Vec<(DynamicImage, String)> {
    let median = |mut x: Vec<u8>| {
        x.sort_unstable(); // sort_unstable é ligeiramente mais rápido
        return x[x.len() / 2];
    };
    let (width, height) = img.dimensions();

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    let cap = (tam as usize) * (tam as usize);

    for pixel in img.pixels() {
        let temp = saida.get_pixel_mut(pixel.0, pixel.1);
        let mut vecrtemp: Vec<u8> = Vec::with_capacity(cap);
        let mut vecgtemp: Vec<u8> = Vec::with_capacity(cap);
        let mut vecbtemp: Vec<u8> = Vec::with_capacity(cap);

        let neighbors = n_neighbors(tam, pixel.0, pixel.1);

        for neighbor in neighbors {
            // CORRIGIDO: >= em vez de > para evitar estouro de limite
            if neighbor.0 < 0
                || neighbor.1 < 0
                || neighbor.0 >= width as i64
                || neighbor.1 >= height as i64
            {
                vecrtemp.push(placeholder);
                vecgtemp.push(placeholder);
                vecbtemp.push(placeholder);
            } else {
                let pixeltemp = img.get_pixel(neighbor.0 as u32, neighbor.1 as u32).0;
                vecrtemp.push(pixeltemp[0]);
                vecgtemp.push(pixeltemp[1]);
                vecbtemp.push(pixeltemp[2]);
            }
        }
        *temp = Rgb([median(vecrtemp), median(vecgtemp), median(vecbtemp)])
    }
    vec.push((DynamicImage::ImageRgb8(saida), "mediana".to_string()));
    vec
}

pub fn filtro_min(img: DynamicImage, tam: u8, placeholder: u8) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    let cap = (tam as usize) * (tam as usize);

    for pixel in img.pixels() {
        let temp = saida.get_pixel_mut(pixel.0, pixel.1);
        let mut vecrtemp: Vec<u8> = Vec::with_capacity(cap);
        let mut vecgtemp: Vec<u8> = Vec::with_capacity(cap);
        let mut vecbtemp: Vec<u8> = Vec::with_capacity(cap);

        let neighbors = n_neighbors(tam, pixel.0, pixel.1);

        for neighbor in neighbors {
            if neighbor.0 < 0
                || neighbor.1 < 0
                || neighbor.0 >= width as i64
                || neighbor.1 >= height as i64
            {
                vecrtemp.push(placeholder);
                vecgtemp.push(placeholder);
                vecbtemp.push(placeholder);
            } else {
                let pixeltemp = img.get_pixel(neighbor.0 as u32, neighbor.1 as u32).0;
                vecrtemp.push(pixeltemp[0]);
                vecgtemp.push(pixeltemp[1]);
                vecbtemp.push(pixeltemp[2]);
            }
        }
        *temp = Rgb([
            *vecrtemp.iter().min().unwrap(),
            *vecgtemp.iter().min().unwrap(),
            *vecbtemp.iter().min().unwrap(),
        ])
    }
    vec.push((DynamicImage::ImageRgb8(saida), "minimo".to_string()));
    vec
}

pub fn filtro_max(img: DynamicImage, tam: u8, placeholder: u8) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    let cap = (tam as usize) * (tam as usize);

    for pixel in img.pixels() {
        let temp = saida.get_pixel_mut(pixel.0, pixel.1);
        let mut vecrtemp: Vec<u8> = Vec::with_capacity(cap);
        let mut vecgtemp: Vec<u8> = Vec::with_capacity(cap);
        let mut vecbtemp: Vec<u8> = Vec::with_capacity(cap);

        let neighbors = n_neighbors(tam, pixel.0, pixel.1);

        for neighbor in neighbors {
            if neighbor.0 < 0
                || neighbor.1 < 0
                || neighbor.0 >= width as i64
                || neighbor.1 >= height as i64
            {
                vecrtemp.push(placeholder);
                vecgtemp.push(placeholder);
                vecbtemp.push(placeholder);
            } else {
                let pixeltemp = img.get_pixel(neighbor.0 as u32, neighbor.1 as u32).0;
                vecrtemp.push(pixeltemp[0]);
                vecgtemp.push(pixeltemp[1]);
                vecbtemp.push(pixeltemp[2]);
            }
        }
        *temp = Rgb([
            *vecrtemp.iter().max().unwrap(),
            *vecgtemp.iter().max().unwrap(),
            *vecbtemp.iter().max().unwrap(),
        ])
    }
    vec.push((DynamicImage::ImageRgb8(saida), "maximo".to_string()));
    vec
}

pub fn equalizacao_histograma(img: DynamicImage) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let img_gray = img.to_luma8();

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(3);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut histograma_antes = [0u32; 256];

    for pixel in img_gray.pixels() {
        histograma_antes[pixel[0] as usize] += 1;
    }

    let mut acumulado = [0u32; 256];
    acumulado[0] = histograma_antes[0];
    for i in 1..256 {
        acumulado[i] = acumulado[i - 1] + histograma_antes[i];
    }

    let total_pixels = (width * height) as f32;
    let mut mapeamento = [0u8; 256];
    for i in 0..256 {
        mapeamento[i] = ((acumulado[i] as f32 / total_pixels) * 255.0).round() as u8;
    }

    let mut histograma_depois = [0u32; 256];
    for (x, y, pixel) in img_gray.enumerate_pixels() {
        let novo_valor = mapeamento[pixel[0] as usize];
        histograma_depois[novo_valor as usize] += 1;
        saida.put_pixel(x, y, Rgb([novo_valor, novo_valor, novo_valor]));
    }

    vec.push((
        DynamicImage::ImageRgb8(saida),
        "Equalizacao de Histograma".to_string(),
    ));
    vec.push((
        DynamicImage::ImageRgb8(desenhar_histograma(&histograma_antes)),
        "Histograma Antes".to_string(),
    ));
    vec.push((
        DynamicImage::ImageRgb8(desenhar_histograma(&histograma_depois)),
        "Histograma Depois".to_string(),
    ));

    vec
}

pub fn fatiamento_intensidade(
    img: DynamicImage,
    fLow: u8,
    fHigh: u8,
    fundo: bool,
) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for pixel in img.pixels() {
        let (x, y) = (pixel.0, pixel.1);
        let rgb = pixel.2;

        let cinza = ((rgb[0] as f32 + rgb[1] as f32 + rgb[2] as f32) / 3.0) as u8;
        let cinza_limiarizado = if cinza >= fLow && cinza <= fHigh {
            255
        } else {
            if fundo { cinza } else { 0 }
        };

        *saida.get_pixel_mut(x, y) = Rgb([cinza_limiarizado, cinza_limiarizado, cinza_limiarizado]);
    }

    vec.push((
        DynamicImage::ImageRgb8(saida),
        "Fatiamento por intensidade".to_string(),
    ));

    vec
}

pub fn media_gaussiana(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
    let kernel = if p.kernel % 2 == 0 {
        p.kernel + 1
    } else {
        p.kernel
    }
    .max(1);
    let sigma = p.sigma.max(0.01);
    let placeholder = crate::valor_placeholder(p.placeholder) as f32;
    let raio = (kernel / 2) as i32;

    let mut nucleo = vec![vec![0.0f32; kernel as usize]; kernel as usize];
    let mut soma_nucleo = 0.0f32;
    for i in 0..kernel as usize {
        for j in 0..kernel as usize {
            let x = (i as i32 - raio) as f32;
            let y = (j as i32 - raio) as f32;
            nucleo[i][j] = (1.0 / (2.0 * std::f32::consts::PI * sigma * sigma))
                * (-(x * x + y * y) / (2.0 * sigma * sigma)).exp();
            soma_nucleo += nucleo[i][j];
        }
    }
    for linha in nucleo.iter_mut() {
        for v in linha.iter_mut() {
            *v /= soma_nucleo;
        }
    }

    let processar_canal = |canal: &image::GrayImage| -> image::GrayImage {
        let (width, height) = canal.dimensions();
        let mut saida = image::ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let mut soma = 0.0f32;
                for ky in 0..kernel as i32 {
                    for kx in 0..kernel as i32 {
                        let px = x as i32 + (kx - raio);
                        let py = y as i32 + (ky - raio);
                        let valor = if px < 0 || py < 0 || px >= width as i32 || py >= height as i32
                        {
                            placeholder
                        } else {
                            canal.get_pixel(px as u32, py as u32)[0] as f32
                        };
                        soma += valor * nucleo[ky as usize][kx as usize];
                    }
                }
                saida.put_pixel(x, y, Luma([soma.round().clamp(0.0, 255.0) as u8]));
            }
        }
        saida
    };

    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }
            let r = processar_canal(&r_buf);
            let g = processar_canal(&g_buf);
            let b = processar_canal(&b_buf);
            let mut saida = ImageBuffer::new(width, height);
            for (x, y, pixel) in saida.enumerate_pixels_mut() {
                *pixel = Rgb([
                    r.get_pixel(x, y)[0],
                    g.get_pixel(x, y)[0],
                    b.get_pixel(x, y)[0],
                ]);
            }
            DynamicImage::ImageRgb8(saida)
        }
        _ => DynamicImage::ImageLuma8(processar_canal(&img.to_luma8())),
    };

    vec![(
        processada,
        format!("Media Gaussiana (kernel={}, sigma={:.2})", kernel, sigma),
    )]
}
fn desenhar_histograma(histograma: &[u32; 256]) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let largura = 256u32;
    let altura = 150u32;
    let mut img = ImageBuffer::from_pixel(largura, altura, Rgb([24u8, 24u8, 37u8]));
    let max = *histograma.iter().max().unwrap_or(&1).max(&1);

    for (i, &contagem) in histograma.iter().enumerate() {
        let altura_barra = ((contagem as f32 / max as f32) * (altura as f32 - 2.0)) as u32;
        for y in 0..altura_barra {
            img.put_pixel(i as u32, altura - 1 - y, Rgb([137, 180, 250]));
        }
    }
    img
}
pub fn filtro_agucamento(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
    let kernel = if p.kernel % 2 == 0 {
        p.kernel + 1
    } else {
        p.kernel
    }
    .max(1);
    let ganho = p.param_1;
    let placeholder = crate::valor_placeholder(p.placeholder) as f32;
    let raio = (kernel / 2) as i32;

    let processar_canal = |canal: &image::GrayImage| -> image::GrayImage {
        let (width, height) = canal.dimensions();
        let mut saida = image::ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let mut soma = 0.0f32;
                let mut n = 0u32;
                for dy in -raio..=raio {
                    for dx in -raio..=raio {
                        let px = x as i32 + dx;
                        let py = y as i32 + dy;
                        let valor = if px < 0 || py < 0 || px >= width as i32 || py >= height as i32
                        {
                            placeholder
                        } else {
                            canal.get_pixel(px as u32, py as u32)[0] as f32
                        };
                        soma += valor;
                        n += 1;
                    }
                }
                let media = soma / n as f32;
                let original = canal.get_pixel(x, y)[0] as f32;
                // Máscara de nitidez: original + ganho * (original - média local)
                let novo = (original + ganho * (original - media)).clamp(0.0, 255.0) as u8;
                saida.put_pixel(x, y, Luma([novo]));
            }
        }
        saida
    };

    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }
            let r = processar_canal(&r_buf);
            let g = processar_canal(&g_buf);
            let b = processar_canal(&b_buf);
            let mut saida = ImageBuffer::new(width, height);
            for (x, y, pixel) in saida.enumerate_pixels_mut() {
                *pixel = Rgb([
                    r.get_pixel(x, y)[0],
                    g.get_pixel(x, y)[0],
                    b.get_pixel(x, y)[0],
                ]);
            }
            DynamicImage::ImageRgb8(saida)
        }
        _ => DynamicImage::ImageLuma8(processar_canal(&img.to_luma8())),
    };

    vec![(
        processada,
        format!(
            "Mascara de Agucamento (kernel={}, ganho={:.2})",
            kernel, ganho
        ),
    )]
}

pub fn agucamento_laplaciano(
    img: DynamicImage,
    p: &crate::Parametros,
) -> Vec<(DynamicImage, String)> {
    let ganho = p.param_1;
    let placeholder = crate::valor_placeholder(p.placeholder) as f32;

    let nucleo_4v: [[f32; 3]; 3] = [[0.0, -1.0, 0.0], [-1.0, 4.0, -1.0], [0.0, -1.0, 0.0]];
    let nucleo_8v: [[f32; 3]; 3] = [[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]];

    let aplicar = |canal: &image::GrayImage,
                   nucleo: &[[f32; 3]; 3]|
     -> (image::GrayImage, image::GrayImage) {
        let (width, height) = canal.dimensions();
        let mut saida = image::ImageBuffer::new(width, height);
        let mut filtro = image::ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let mut soma = 0.0f32;
                for ky in 0..3i32 {
                    for kx in 0..3i32 {
                        let px = x as i32 + (kx - 1);
                        let py = y as i32 + (ky - 1);
                        let valor = if px < 0 || py < 0 || px >= width as i32 || py >= height as i32
                        {
                            placeholder
                        } else {
                            canal.get_pixel(px as u32, py as u32)[0] as f32
                        };
                        soma += valor * nucleo[ky as usize][kx as usize];
                    }
                }
                let original = canal.get_pixel(x, y)[0] as f32;
                filtro.put_pixel(x, y, Luma([soma.clamp(0.0, 255.0) as u8]));
                let novo = (original + ganho * soma).clamp(0.0, 255.0) as u8;
                saida.put_pixel(x, y, Luma([novo]));
            }
        }
        (saida, filtro)
    };

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(4);

    match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }

            for (nucleo, rotulo) in [(&nucleo_4v, "4vizinhancas"), (&nucleo_8v, "8vizinhancas")] {
                let (r_saida, r_filtro) = aplicar(&r_buf, nucleo);
                let (g_saida, g_filtro) = aplicar(&g_buf, nucleo);
                let (b_saida, b_filtro) = aplicar(&b_buf, nucleo);

                let mut saida_rgb = ImageBuffer::new(width, height);
                let mut filtro_rgb = ImageBuffer::new(width, height);
                for y in 0..height {
                    for x in 0..width {
                        saida_rgb.put_pixel(
                            x,
                            y,
                            Rgb([
                                r_saida.get_pixel(x, y)[0],
                                g_saida.get_pixel(x, y)[0],
                                b_saida.get_pixel(x, y)[0],
                            ]),
                        );
                        filtro_rgb.put_pixel(
                            x,
                            y,
                            Rgb([
                                r_filtro.get_pixel(x, y)[0],
                                g_filtro.get_pixel(x, y)[0],
                                b_filtro.get_pixel(x, y)[0],
                            ]),
                        );
                    }
                }
                vec.push((
                    DynamicImage::ImageRgb8(filtro_rgb),
                    format!("Filtro Laplaciano de {}", rotulo),
                ));
                vec.push((
                    DynamicImage::ImageRgb8(saida_rgb),
                    format!("Agucamento Laplaciano de {}", rotulo),
                ));
            }
        }
        _ => {
            let gray = img.to_luma8();
            for (nucleo, rotulo) in [(&nucleo_4v, "4vizinhancas"), (&nucleo_8v, "8vizinhancas")] {
                let (saida, filtro) = aplicar(&gray, nucleo);
                vec.push((
                    DynamicImage::ImageLuma8(filtro),
                    format!("Filtro Laplaciano de {}", rotulo),
                ));
                vec.push((
                    DynamicImage::ImageLuma8(saida),
                    format!("Agucamento Laplaciano de {}", rotulo),
                ));
            }
        }
    }

    vec
}
pub fn agucamento_sobel(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
    let fator = p.param_1;
    let placeholder = crate::valor_placeholder(p.placeholder) as f32;

    let sobel_x = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];
    let sobel_y = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];

    let processar = |canal: &image::GrayImage| -> (image::GrayImage, image::GrayImage) {
        let (width, height) = canal.dimensions();
        let mut gradiente = vec![vec![0.0f32; width as usize]; height as usize];
        let mut max_gradiente = 0.0f32;

        for y in 0..height {
            for x in 0..width {
                let mut gx = 0.0f32;
                let mut gy = 0.0f32;
                for ky in 0..3i32 {
                    for kx in 0..3i32 {
                        let px = x as i32 + (kx - 1);
                        let py = y as i32 + (ky - 1);
                        let valor = if px < 0 || py < 0 || px >= width as i32 || py >= height as i32
                        {
                            placeholder
                        } else {
                            canal.get_pixel(px as u32, py as u32)[0] as f32
                        };
                        gx += valor * sobel_x[ky as usize][kx as usize];
                        gy += valor * sobel_y[ky as usize][kx as usize];
                    }
                }
                let magnitude = (gx * gx + gy * gy).sqrt();
                gradiente[y as usize][x as usize] = magnitude;
                if magnitude > max_gradiente {
                    max_gradiente = magnitude;
                }
            }
        }

        let mut saida = image::ImageBuffer::new(width, height);
        let mut filtro = image::ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let original = canal.get_pixel(x, y)[0] as f32;
                let grad_norm = if max_gradiente > 0.0 {
                    gradiente[y as usize][x as usize] / max_gradiente
                } else {
                    0.0
                };
                let novo = (original + fator * grad_norm * 255.0).clamp(0.0, 255.0) as u8;
                let grad_u8 = (grad_norm * 255.0).clamp(0.0, 255.0) as u8;
                saida.put_pixel(x, y, Luma([novo]));
                filtro.put_pixel(x, y, Luma([grad_u8]));
            }
        }
        (saida, filtro)
    };

    let (processada, filtro_vis) = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }
            let (r_saida, r_filtro) = processar(&r_buf);
            let (g_saida, g_filtro) = processar(&g_buf);
            let (b_saida, b_filtro) = processar(&b_buf);

            let mut saida_rgb = ImageBuffer::new(width, height);
            let mut filtro_rgb = ImageBuffer::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    saida_rgb.put_pixel(
                        x,
                        y,
                        Rgb([
                            r_saida.get_pixel(x, y)[0],
                            g_saida.get_pixel(x, y)[0],
                            b_saida.get_pixel(x, y)[0],
                        ]),
                    );
                    filtro_rgb.put_pixel(
                        x,
                        y,
                        Rgb([
                            r_filtro.get_pixel(x, y)[0],
                            g_filtro.get_pixel(x, y)[0],
                            b_filtro.get_pixel(x, y)[0],
                        ]),
                    );
                }
            }
            (
                DynamicImage::ImageRgb8(saida_rgb),
                DynamicImage::ImageRgb8(filtro_rgb),
            )
        }
        _ => {
            let gray = img.to_luma8();
            let (saida, filtro) = processar(&gray);
            (
                DynamicImage::ImageLuma8(saida),
                DynamicImage::ImageLuma8(filtro),
            )
        }
    };

    vec![
        (
            processada,
            "Imagem com agucamento por gradiente de Sobel".to_string(),
        ),
        (
            filtro_vis,
            "Filtro de agucamento por gradiente de Sobel (gradiente)".to_string(),
        ),
    ]
}

pub fn passa_baixa_gaussiano(
    img: DynamicImage,
    p: &crate::Parametros,
) -> Vec<(DynamicImage, String)> {
    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            // Separa os canais R, G e B, aplica o filtro em cada um e recompõe o RGB
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);

            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }

            let r_filtrado = aplicar_filtro_gaussiano_frequencia(
                &DynamicImage::ImageLuma8(r_buf),
                p.freq_corte,
                false,
            );
            let g_filtrado = aplicar_filtro_gaussiano_frequencia(
                &DynamicImage::ImageLuma8(g_buf),
                p.freq_corte,
                false,
            );
            let b_filtrado = aplicar_filtro_gaussiano_frequencia(
                &DynamicImage::ImageLuma8(b_buf),
                p.freq_corte,
                false,
            );

            let r_gray = r_filtrado.to_luma8();
            let g_gray = g_filtrado.to_luma8();
            let b_gray = b_filtrado.to_luma8();

            let mut rgb_saida = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_saida.enumerate_pixels_mut() {
                *pixel = Rgb([
                    r_gray.get_pixel(x, y)[0],
                    g_gray.get_pixel(x, y)[0],
                    b_gray.get_pixel(x, y)[0],
                ]);
            }
            DynamicImage::ImageRgb8(rgb_saida)
        }
        _ => {
            // Se for escala de cinza, aplica direto
            aplicar_filtro_gaussiano_frequencia(&img, p.freq_corte, false)
        }
    };

    vec![(
        processada,
        format!("Gaussiano Passa-Baixa (D0={:.1})", p.freq_corte),
    )]
}

pub fn passa_alta_gaussiano(
    img: DynamicImage,
    p: &crate::Parametros,
) -> Vec<(DynamicImage, String)> {
    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);

            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }

            let r_filtrado = aplicar_filtro_gaussiano_frequencia(
                &DynamicImage::ImageLuma8(r_buf),
                p.freq_corte,
                true,
            );
            let g_filtrado = aplicar_filtro_gaussiano_frequencia(
                &DynamicImage::ImageLuma8(g_buf),
                p.freq_corte,
                true,
            );
            let b_filtrado = aplicar_filtro_gaussiano_frequencia(
                &DynamicImage::ImageLuma8(b_buf),
                p.freq_corte,
                true,
            );

            let r_gray = r_filtrado.to_luma8();
            let g_gray = g_filtrado.to_luma8();
            let b_gray = b_filtrado.to_luma8();

            let mut rgb_saida = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_saida.enumerate_pixels_mut() {
                *pixel = Rgb([
                    r_gray.get_pixel(x, y)[0],
                    g_gray.get_pixel(x, y)[0],
                    b_gray.get_pixel(x, y)[0],
                ]);
            }
            DynamicImage::ImageRgb8(rgb_saida)
        }
        _ => aplicar_filtro_gaussiano_frequencia(&img, p.freq_corte, true),
    };

    vec![(
        processada,
        format!("Gaussiano Passa-Alta (D0={:.1})", p.freq_corte),
    )]
}

/// Função central que faz a FFT, aplica a máscara e faz a IFFT
fn aplicar_filtro_gaussiano_frequencia(
    img: &DynamicImage,
    freq_corte: f32,
    passa_alta: bool,
) -> DynamicImage {
    // Converte a imagem para tons de cinza (processamento padrão em frequência)
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();
    let w = width as usize;
    let h = height as usize;

    // 1. Preparar os dados espaciais com o truque do shift:
    // Multiplicar por (-1)^(x+y) centraliza as baixas frequências no meio da imagem na FFT
    let mut data = vec![Complex::new(0.0, 0.0); w * h];
    for y in 0..h {
        for x in 0..w {
            let pixel = gray.get_pixel(x as u32, y as u32)[0] as f32;
            let sign = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
            data[y * w + x] = Complex::new(pixel * sign, 0.0);
        }
    }

    let mut planner = FftPlanner::new();

    // 2. FFT 2D (Forward) - Aplica a FFT nas linhas e depois nas colunas
    let fft_row = planner.plan_fft_forward(w);
    for row in data.chunks_mut(w) {
        fft_row.process(row);
    }

    // Transpõe para processar colunas como se fossem linhas
    let mut data_t = transpor_matriz(&data, w, h);
    let fft_col = planner.plan_fft_forward(h);
    for row in data_t.chunks_mut(h) {
        fft_col.process(row);
    }
    // Transpõe de volta para o layout original
    data = transpor_matriz(&data_t, h, w);

    // 3. Aplicar a Máscara Gaussiana no domínio da frequência
    let center_u = (w / 2) as f32;
    let center_v = (h / 2) as f32;
    // Evita divisão por zero se o usuário botar 0 no slider
    let d0_sq = freq_corte.max(0.1) * freq_corte.max(0.1);

    for y in 0..h {
        for x in 0..w {
            let du = x as f32 - center_u;
            let dv = y as f32 - center_v;
            let d_sq = du * du + dv * dv;

            // Fórmula do Filtro Gaussiano: e^(-D^2 / 2*D0^2)
            let mut mask = (-d_sq / (2.0 * d0_sq)).exp();

            if passa_alta {
                mask = 1.0 - mask;
            }

            // Multiplica o número complexo pela máscara (que é um valor real de 0.0 a 1.0)
            data[y * w + x] *= mask;
        }
    }

    // 4. IFFT 2D (Inverse) - Traz a imagem de volta para o domínio espacial
    let ifft_row = planner.plan_fft_inverse(w);
    for row in data.chunks_mut(w) {
        ifft_row.process(row);
    }

    let mut data_t = transpor_matriz(&data, w, h);
    let ifft_col = planner.plan_fft_inverse(h);
    for row in data_t.chunks_mut(h) {
        ifft_col.process(row);
    }
    data = transpor_matriz(&data_t, h, w);

    // 5. Reconstruir a imagem espacial
    let mut out_img = ImageBuffer::new(width, height);
    let normalizer = (w * h) as f32; // A IFFT da biblioteca rustfft não normaliza automaticamente

    for y in 0..h {
        for x in 0..w {
            let sign = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };

            // Pega a parte real, reverte o shift inicial e divide pela normalização
            let val = (data[y * w + x].re / normalizer) * sign;

            // Garante que o valor fique entre 0 e 255
            let pixel_val = val.clamp(0.0, 255.0) as u8;
            out_img.put_pixel(x as u32, y as u32, Luma([pixel_val]));
        }
    }

    DynamicImage::ImageLuma8(out_img)
}

/// Helper: Como as bibliotecas de FFT operam em arrays 1D sequenciais, usamos
/// isso para transpor linhas/colunas de forma eficiente durante a FFT 2D.
fn transpor_matriz(data: &[Complex<f32>], width: usize, height: usize) -> Vec<Complex<f32>> {
    let mut out = vec![Complex::new(0.0, 0.0); data.len()];
    for y in 0..height {
        for x in 0..width {
            out[x * height + y] = data[y * width + x];
        }
    }
    out
}

pub fn passa_baixa_butterworth(
    img: DynamicImage,
    p: &crate::Parametros,
) -> Vec<(DynamicImage, String)> {
    let ordem = p.ordem.max(1) as f32;
    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);

            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }

            let r_filtrado = aplicar_filtro_butterworth_frequencia(
                &DynamicImage::ImageLuma8(r_buf),
                p.freq_corte,
                ordem,
                false,
            );
            let g_filtrado = aplicar_filtro_butterworth_frequencia(
                &DynamicImage::ImageLuma8(g_buf),
                p.freq_corte,
                ordem,
                false,
            );
            let b_filtrado = aplicar_filtro_butterworth_frequencia(
                &DynamicImage::ImageLuma8(b_buf),
                p.freq_corte,
                ordem,
                false,
            );

            let r_gray = r_filtrado.to_luma8();
            let g_gray = g_filtrado.to_luma8();
            let b_gray = b_filtrado.to_luma8();

            let mut rgb_saida = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_saida.enumerate_pixels_mut() {
                *pixel = Rgb([
                    r_gray.get_pixel(x, y)[0],
                    g_gray.get_pixel(x, y)[0],
                    b_gray.get_pixel(x, y)[0],
                ]);
            }
            DynamicImage::ImageRgb8(rgb_saida)
        }
        _ => aplicar_filtro_butterworth_frequencia(&img, p.freq_corte, ordem, false),
    };

    vec![(
        processada,
        format!(
            "Butterworth Passa-Baixa (D0={:.1}, n={:.0})",
            p.freq_corte, ordem
        ),
    )]
}

pub fn passa_alta_butterworth(
    img: DynamicImage,
    p: &crate::Parametros,
) -> Vec<(DynamicImage, String)> {
    let ordem = p.ordem.max(1) as f32;
    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);

            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }

            let r_filtrado = aplicar_filtro_butterworth_frequencia(
                &DynamicImage::ImageLuma8(r_buf),
                p.freq_corte,
                ordem,
                true,
            );
            let g_filtrado = aplicar_filtro_butterworth_frequencia(
                &DynamicImage::ImageLuma8(g_buf),
                p.freq_corte,
                ordem,
                true,
            );
            let b_filtrado = aplicar_filtro_butterworth_frequencia(
                &DynamicImage::ImageLuma8(b_buf),
                p.freq_corte,
                ordem,
                true,
            );

            let r_gray = r_filtrado.to_luma8();
            let g_gray = g_filtrado.to_luma8();
            let b_gray = b_filtrado.to_luma8();

            let mut rgb_saida = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_saida.enumerate_pixels_mut() {
                *pixel = Rgb([
                    r_gray.get_pixel(x, y)[0],
                    g_gray.get_pixel(x, y)[0],
                    b_gray.get_pixel(x, y)[0],
                ]);
            }
            DynamicImage::ImageRgb8(rgb_saida)
        }
        _ => aplicar_filtro_butterworth_frequencia(&img, p.freq_corte, ordem, true),
    };

    vec![(
        processada,
        format!(
            "Butterworth Passa-Alta (D0={:.1}, n={:.0})",
            p.freq_corte, ordem
        ),
    )]
}

fn aplicar_filtro_butterworth_frequencia(
    img: &DynamicImage,
    freq_corte: f32,
    ordem: f32,
    passa_alta: bool,
) -> DynamicImage {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();
    let w = width as usize;
    let h = height as usize;

    let mut data = vec![Complex::new(0.0, 0.0); w * h];
    for y in 0..h {
        for x in 0..w {
            let pixel = gray.get_pixel(x as u32, y as u32)[0] as f32;
            let sign = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
            data[y * w + x] = Complex::new(pixel * sign, 0.0);
        }
    }

    let mut planner = FftPlanner::new();

    let fft_row = planner.plan_fft_forward(w);
    for row in data.chunks_mut(w) {
        fft_row.process(row);
    }

    let mut data_t = transpor_matriz(&data, w, h);
    let fft_col = planner.plan_fft_forward(h);
    for row in data_t.chunks_mut(h) {
        fft_col.process(row);
    }
    data = transpor_matriz(&data_t, h, w);

    let center_u = (w / 2) as f32;
    let center_v = (h / 2) as f32;
    let d0 = freq_corte.max(0.1);

    for y in 0..h {
        for x in 0..w {
            let du = x as f32 - center_u;
            let dv = y as f32 - center_v;
            let d = (du * du + dv * dv).sqrt();

            // Fórmula do Filtro Butterworth de Ordem n: 1 / (1 + (D/D0)^(2n))
            let mut mask = 1.0 / (1.0 + (d / d0).powf(2.0 * ordem));

            if passa_alta {
                mask = 1.0 - mask;
            }

            data[y * w + x] *= mask;
        }
    }

    let ifft_row = planner.plan_fft_inverse(w);
    for row in data.chunks_mut(w) {
        ifft_row.process(row);
    }

    let mut data_t = transpor_matriz(&data, w, h);
    let ifft_col = planner.plan_fft_inverse(h);
    for row in data_t.chunks_mut(h) {
        ifft_col.process(row);
    }
    data = transpor_matriz(&data_t, h, w);

    let mut out_img = ImageBuffer::new(width, height);
    let normalizer = (w * h) as f32;

    for y in 0..h {
        for x in 0..w {
            let sign = if (x + y) % 2 == 0 { 1.0 } else { -1.0 };
            let val = (data[y * w + x].re / normalizer) * sign;
            let pixel_val = val.clamp(0.0, 255.0) as u8;
            out_img.put_pixel(x as u32, y as u32, Luma([pixel_val]));
        }
    }

    DynamicImage::ImageLuma8(out_img)
}

/// 1. Filtro Adaptativo de Mediana (Suporta RGB e tamanho máximo de janela via p.kernel)
pub fn filtro_adaptativo_mediana(
    img: DynamicImage,
    p: &crate::Parametros,
) -> Vec<(DynamicImage, String)> {
    let max_kernel = p.kernel.max(3) as i32;

    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut r_buf = ImageBuffer::new(width, height);
            let mut g_buf = ImageBuffer::new(width, height);
            let mut b_buf = ImageBuffer::new(width, height);

            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                r_buf.put_pixel(x, y, Luma([pixel[0]]));
                g_buf.put_pixel(x, y, Luma([pixel[1]]));
                b_buf.put_pixel(x, y, Luma([pixel[2]]));
            }

            let r_filtrado = aplicar_adaptativo_mediana_canal(&r_buf, max_kernel);
            let g_filtrado = aplicar_adaptativo_mediana_canal(&g_buf, max_kernel);
            let b_filtrado = aplicar_adaptativo_mediana_canal(&b_buf, max_kernel);

            let mut rgb_saida = ImageBuffer::new(width, height);
            for (x, y, pixel) in rgb_saida.enumerate_pixels_mut() {
                *pixel = Rgb([
                    r_filtrado.get_pixel(x, y)[0],
                    g_filtrado.get_pixel(x, y)[0],
                    b_filtrado.get_pixel(x, y)[0],
                ]);
            }
            DynamicImage::ImageRgb8(rgb_saida)
        }
        _ => {
            let gray = img.to_luma8();
            DynamicImage::ImageLuma8(aplicar_adaptativo_mediana_canal(&gray, max_kernel))
        }
    };

    vec![(
        processada,
        format!("Filtro Adaptativo Mediana (Max S={})", max_kernel),
    )]
}

/// 2. Ruído Aditivo Gaussiano (Suporta RGB)
pub fn ruido_aditivo_gaussiano(
    img: DynamicImage,
    p: &crate::Parametros,
) -> Vec<(DynamicImage, String)> {
    let mut rng = rand::thread_rng();
    let desvio_padrao = (p.param_1 * 5.0).max(1.0) as f64;

    let processada = match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut saida = ImageBuffer::new(width, height);

            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                let mut novo_pixel = [0u8; 3];
                for i in 0..3 {
                    let val_canal = pixel[i] as f64;
                    let r1: f64 = rng.gen_range(0.0..1.0);
                    let r2: f64 = rng.gen_range(0.0..1.0);
                    let r3: f64 = rng.gen_range(0.0..1.0);
                    let normal_approx = (r1 + r2 + r3 - 1.5) * 2.0 * desvio_padrao;
                    novo_pixel[i] = (val_canal + normal_approx).clamp(0.0, 255.0) as u8;
                }
                saida.put_pixel(x, y, Rgb(novo_pixel));
            }
            DynamicImage::ImageRgb8(saida)
        }
        _ => {
            let gray = img.to_luma8();
            let (width, height) = gray.dimensions();
            let mut saida = ImageBuffer::new(width, height);

            for y in 0..height {
                for x in 0..width {
                    let pixel = gray.get_pixel(x, y)[0] as f64;
                    let r1: f64 = rng.gen_range(0.0..1.0);
                    let r2: f64 = rng.gen_range(0.0..1.0);
                    let r3: f64 = rng.gen_range(0.0..1.0);
                    let normal_approx = (r1 + r2 + r3 - 1.5) * 2.0 * desvio_padrao;
                    let val = (pixel + normal_approx).clamp(0.0, 255.0) as u8;
                    saida.put_pixel(x, y, Luma([val]));
                }
            }
            DynamicImage::ImageLuma8(saida)
        }
    };

    vec![(
        processada,
        format!("Ruído Aditivo Gaussiano (Desvio={:.1})", desvio_padrao),
    )]
}

/// Helper interno para aplicar o algoritmo do Filtro Adaptativo de Mediana em um único canal (Luma8)
fn aplicar_adaptativo_mediana_canal(gray: &image::GrayImage, max_kernel: i32) -> image::GrayImage {
    let (width, height) = gray.dimensions();
    let mut saida = image::ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut k = 3;
            let mut val_final = gray.get_pixel(x, y)[0];

            while k <= max_kernel {
                let mut vizinhos = Vec::new();
                let raio = k / 2;

                for dy in -raio..=raio {
                    for dx in -raio..=raio {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            vizinhos.push(gray.get_pixel(nx as u32, ny as u32)[0]);
                        }
                    }
                }

                vizinhos.sort();
                let min = *vizinhos.first().unwrap();
                let max = *vizinhos.last().unwrap();
                let med = vizinhos[vizinhos.len() / 2];
                let z_xy = gray.get_pixel(x, y)[0];

                let a1 = z_xy as i32 - min as i32;
                let a2 = z_xy as i32 - max as i32;
                if a1 > 0 && a2 < 0 {
                    let b1 = med as i32 - min as i32;
                    let b2 = med as i32 - max as i32;
                    if b1 > 0 && b2 < 0 {
                        val_final = z_xy;
                        break;
                    } else {
                        val_final = med;
                        break;
                    }
                } else {
                    k += 2;
                    if k > max_kernel {
                        val_final = med;
                    }
                }
            }
            saida.put_pixel(x, y, Luma([val_final]));
        }
    }
    saida
}

/// Aplicação de ruído Sal, Pimenta, Sal e Pimenta com suporte a RGB e ajuste de distribuição
/// Aplicação de ruído Sal, Pimenta e Sal-e-Pimenta.
/// `distribuicao` (p.distribuicao_ruido) é a probabilidade (0.0 a 1.0) de
/// um pixel ser corrompido. No caso combinado, essa probabilidade é
/// dividida ao meio entre sal e pimenta.
pub fn ruido_sal_pimenta(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
    let mut rng = rand::thread_rng();
    let distribuicao = p.distribuicao_ruido.clamp(0.0, 1.0);

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(3);

    match &img {
        DynamicImage::ImageRgb8(rgb_img) => {
            let (width, height) = rgb_img.dimensions();
            let mut saida_sal = rgb_img.clone();
            let mut saida_pimenta = rgb_img.clone();
            let mut saida_sal_pimenta = rgb_img.clone();

            for y in 0..height {
                for x in 0..width {
                    let r1: f32 = rng.r#gen();
                    if r1 < distribuicao {
                        saida_sal.put_pixel(x, y, Rgb([255, 255, 255]));
                    }

                    let r2: f32 = rng.r#gen();
                    if r2 < distribuicao {
                        saida_pimenta.put_pixel(x, y, Rgb([0, 0, 0]));
                    }

                    let r3: f32 = rng.r#gen();
                    if r3 < distribuicao / 2.0 {
                        saida_sal_pimenta.put_pixel(x, y, Rgb([255, 255, 255]));
                    } else if r3 < distribuicao {
                        saida_sal_pimenta.put_pixel(x, y, Rgb([0, 0, 0]));
                    }
                }
            }

            vec.push((DynamicImage::ImageRgb8(saida_sal), "Ruido_Sal".to_string()));
            vec.push((
                DynamicImage::ImageRgb8(saida_pimenta),
                "Ruido_Pimenta".to_string(),
            ));
            vec.push((
                DynamicImage::ImageRgb8(saida_sal_pimenta),
                "Ruido_Sal_e_Pimenta".to_string(),
            ));
        }
        _ => {
            let gray = img.to_luma8();
            let (width, height) = gray.dimensions();
            let mut saida_sal = gray.clone();
            let mut saida_pimenta = gray.clone();
            let mut saida_sal_pimenta = gray.clone();

            for y in 0..height {
                for x in 0..width {
                    let r1: f32 = rng.r#gen();
                    if r1 < distribuicao {
                        saida_sal.put_pixel(x, y, Luma([255]));
                    }

                    let r2: f32 = rng.r#gen();
                    if r2 < distribuicao {
                        saida_pimenta.put_pixel(x, y, Luma([0]));
                    }

                    let r3: f32 = rng.r#gen();
                    if r3 < distribuicao / 2.0 {
                        saida_sal_pimenta.put_pixel(x, y, Luma([255]));
                    } else if r3 < distribuicao {
                        saida_sal_pimenta.put_pixel(x, y, Luma([0]));
                    }
                }
            }

            vec.push((DynamicImage::ImageLuma8(saida_sal), "Ruido_Sal".to_string()));
            vec.push((
                DynamicImage::ImageLuma8(saida_pimenta),
                "Ruido_Pimenta".to_string(),
            ));
            vec.push((
                DynamicImage::ImageLuma8(saida_sal_pimenta),
                "Ruido_Sal_e_Pimenta".to_string(),
            ));
        }
    }

    vec
}
