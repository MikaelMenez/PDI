use image::*;
use std::error::Error;
use image::{DynamicImage, ImageBuffer, Luma};
use rustfft::{num_complex::Complex, FftPlanner};
use rand::Rng;

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
    let transform = |x| (ganho * ((x as f32) / 255_f32).powf(gama)) as u8;
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

    let cap = (tam * tam) as usize;

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

    let cap = (tam * tam) as usize;

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

    let cap = (tam * tam) as usize;

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

pub fn passa_baixa_gaussiano(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
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

            let r_filtrado = aplicar_filtro_gaussiano_frequencia(&DynamicImage::ImageLuma8(r_buf), p.freq_corte, false);
            let g_filtrado = aplicar_filtro_gaussiano_frequencia(&DynamicImage::ImageLuma8(g_buf), p.freq_corte, false);
            let b_filtrado = aplicar_filtro_gaussiano_frequencia(&DynamicImage::ImageLuma8(b_buf), p.freq_corte, false);

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

    vec![(processada, format!("Gaussiano Passa-Baixa (D0={:.1})", p.freq_corte))]
}

pub fn passa_alta_gaussiano(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
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

            let r_filtrado = aplicar_filtro_gaussiano_frequencia(&DynamicImage::ImageLuma8(r_buf), p.freq_corte, true);
            let g_filtrado = aplicar_filtro_gaussiano_frequencia(&DynamicImage::ImageLuma8(g_buf), p.freq_corte, true);
            let b_filtrado = aplicar_filtro_gaussiano_frequencia(&DynamicImage::ImageLuma8(b_buf), p.freq_corte, true);

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
            aplicar_filtro_gaussiano_frequencia(&img, p.freq_corte, true)
        }
    };

    vec![(processada, format!("Gaussiano Passa-Alta (D0={:.1})", p.freq_corte))]
}

/// Função central que faz a FFT, aplica a máscara e faz a IFFT
fn aplicar_filtro_gaussiano_frequencia(img: &DynamicImage, freq_corte: f32, passa_alta: bool) -> DynamicImage {
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

pub fn passa_baixa_butterworth(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
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

            let r_filtrado = aplicar_filtro_butterworth_frequencia(&DynamicImage::ImageLuma8(r_buf), p.freq_corte, ordem, false);
            let g_filtrado = aplicar_filtro_butterworth_frequencia(&DynamicImage::ImageLuma8(g_buf), p.freq_corte, ordem, false);
            let b_filtrado = aplicar_filtro_butterworth_frequencia(&DynamicImage::ImageLuma8(b_buf), p.freq_corte, ordem, false);

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
            aplicar_filtro_butterworth_frequencia(&img, p.freq_corte, ordem, false)
        }
    };

    vec![(processada, format!("Butterworth Passa-Baixa (D0={:.1}, n={:.0})", p.freq_corte, ordem))]
}

pub fn passa_alta_butterworth(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
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

            let r_filtrado = aplicar_filtro_butterworth_frequencia(&DynamicImage::ImageLuma8(r_buf), p.freq_corte, ordem, true);
            let g_filtrado = aplicar_filtro_butterworth_frequencia(&DynamicImage::ImageLuma8(g_buf), p.freq_corte, ordem, true);
            let b_filtrado = aplicar_filtro_butterworth_frequencia(&DynamicImage::ImageLuma8(b_buf), p.freq_corte, ordem, true);

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
            aplicar_filtro_butterworth_frequencia(&img, p.freq_corte, ordem, true)
        }
    };

    vec![(processada, format!("Butterworth Passa-Alta (D0={:.1}, n={:.0})", p.freq_corte, ordem))]
}

fn aplicar_filtro_butterworth_frequencia(img: &DynamicImage, freq_corte: f32, ordem: f32, passa_alta: bool) -> DynamicImage {
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
pub fn filtro_adaptativo_mediana(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
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

    vec![(processada, format!("Filtro Adaptativo Mediana (Max S={})", max_kernel))]
}

/// 2. Ruído Aditivo Gaussiano (Suporta RGB)
pub fn ruido_aditivo_gaussiano(img: DynamicImage, p: &crate::Parametros) -> Vec<(DynamicImage, String)> {
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

    vec![(processada, format!("Ruído Aditivo Gaussiano (Desvio={:.1})", desvio_padrao))]
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