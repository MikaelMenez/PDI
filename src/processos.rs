use image::*;
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

/// Aplica a transformação de potência (Correção Gamma).
/// Fórmula normalizada: s = 255 * (r / 255) ^ gamma
pub fn transformacao_potencia(img: DynamicImage, gama: f32) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(8);

    let mut cinza_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut cinza_gama_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut potencia_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut potencia_gama_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    let mut cinza_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut cinza_gama_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut potencia_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut potencia_gama_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for pixel in img.pixels() {
        let (x, y) = (pixel.0, pixel.1);
        let rgb = pixel.2;

        // --- Processamento Simples ---
        let int_base_f32 = (rgb[0] as f32 + rgb[1] as f32 + rgb[2] as f32) / 3.0;
        let int_base = int_base_f32 as u8;

        let int_gama_base =
            (255.0 * (int_base_f32 / 255.0).powf(1.0 / gama)).clamp(0.0, 255.0) as u8;
        let int_potencia_base = (255.0 * (int_base_f32 / 255.0).powf(gama)).clamp(0.0, 255.0) as u8;
        let int_potencia_gama_base =
            (255.0 * ((int_gama_base as f32) / 255.0).powf(gama)).clamp(0.0, 255.0) as u8;

        // --- Processamento Perceptual (Olho) ---
        let int_olho_f32 = rgb[0] as f32 * 0.2126 + rgb[1] as f32 * 0.7152 + rgb[2] as f32 * 0.0722;
        let int_olho = int_olho_f32 as u8;

        let int_gama_olho =
            (255.0 * (int_olho_f32 / 255.0).powf(1.0 / gama)).clamp(0.0, 255.0) as u8;
        let int_potencia_olho = (255.0 * (int_olho_f32 / 255.0).powf(gama)).clamp(0.0, 255.0) as u8;
        let int_potencia_gama_olho =
            (255.0 * ((int_gama_olho as f32) / 255.0).powf(gama)).clamp(0.0, 255.0) as u8;

        // Escrita nos Buffers Simples
        *cinza_base.get_pixel_mut(x, y) = Rgb([int_base, int_base, int_base]);
        *cinza_gama_base.get_pixel_mut(x, y) = Rgb([int_gama_base, int_gama_base, int_gama_base]);
        *potencia_base.get_pixel_mut(x, y) =
            Rgb([int_potencia_base, int_potencia_base, int_potencia_base]);
        *potencia_gama_base.get_pixel_mut(x, y) = Rgb([
            int_potencia_gama_base,
            int_potencia_gama_base,
            int_potencia_gama_base,
        ]);

        // Escrita nos Buffers Perceptuais
        *cinza_olho.get_pixel_mut(x, y) = Rgb([int_olho, int_olho, int_olho]);
        *cinza_gama_olho.get_pixel_mut(x, y) = Rgb([int_gama_olho, int_gama_olho, int_gama_olho]);
        *potencia_olho.get_pixel_mut(x, y) =
            Rgb([int_potencia_olho, int_potencia_olho, int_potencia_olho]);
        *potencia_gama_olho.get_pixel_mut(x, y) = Rgb([
            int_potencia_gama_olho,
            int_potencia_gama_olho,
            int_potencia_gama_olho,
        ]);
    }

    vec.push((
        cinza_base.into(),
        "Imagem_Escala_De_Cinza_Simples".to_string(),
    ));
    vec.push((
        cinza_gama_base.into(),
        "Imagem_Gama_Inverso_Simples".to_string(),
    ));
    vec.push((
        potencia_base.into(),
        "Transformacao_Potencia_Simples".to_string(),
    ));
    vec.push((
        potencia_gama_base.into(),
        "Transformacao_Potencia_Gama_Simples".to_string(),
    ));

    vec.push((
        cinza_olho.into(),
        "Imagem_Escala_De_Cinza_Adaptada".to_string(),
    ));
    vec.push((
        cinza_gama_olho.into(),
        "Imagem_Gama_Inverso_Adaptada".to_string(),
    ));
    vec.push((
        potencia_olho.into(),
        "Transformacao_Potencia_Adaptada".to_string(),
    ));
    vec.push((
        potencia_gama_olho.into(),
        "Transformacao_Potencia_Gama_Adaptada".to_string(),
    ));

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
