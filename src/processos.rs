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

pub fn limiarizacao(img: image::DynamicImage, limiar: u8) -> Vec<(DynamicImage, String)> {
    let mut vec: Vec<(DynamicImage, String)> = vec![];
    let mut cinza_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut binaria_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut cinza_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut binaria_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    for pixel in img.pixels() {
        let intensidade_base = (pixel.2[0] as f32 + pixel.2[1] as f32 + pixel.2[2] as f32) / 3.0;
        let valor_base = intensidade_base as u8;
        let intensidade_olho = pixel.2[0] as f32 * 0.2126 + pixel.2[1] as f32 * 0.7152 + pixel.2[2] as f32 * 0.0722;
        let valor_olho = intensidade_olho as u8;
        let binario_base = if valor_base >= limiar {255} else {0};
        let binario_olho = if valor_olho >= limiar {255} else {0};

        let tempcin_b = cinza_base.get_pixel_mut(pixel.0, pixel.1);
        *tempcin_b = Rgb([valor_base, valor_base, valor_base]);
        let tempbin_b = binaria_base.get_pixel_mut(pixel.0, pixel.1);
        *tempbin_b = Rgb([binario_base, binario_base, binario_base]);
        let tempcin_o = binaria_base.get_pixel_mut(pixel.0, pixel.1);
        *tempcin_o = Rgb([valor_olho, valor_olho, valor_olho]);
        let tempbin_o = binaria_base.get_pixel_mut(pixel.0, pixel.1);
        *tempbin_o = Rgb([binario_olho, binario_olho, binario_olho]);
    }

    vec.push((cinza_base.into(), "Imagem_Escala_De_Cinza_Simples".to_string()));
    vec.push((binaria_base.into(), "Imagem_Limiarizada_Simples".to_string()));
    vec.push((cinza_olho.into(), "Imagem_Escala_De_Cinza_Adaptada".to_string()));
    vec.push((binaria_olho.into(), "Imagem_Limiarizada_Adaptada".to_string()));

    vec
}

pub fn tranformacao_log(img: image::DynamicImage, ganho: f32) -> Vec<(DynamicImage, String)> {
    let mut vec: Vec<(DynamicImage, String)> = vec![];
    let mut cinza_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut ln_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut log10_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut log2_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());

    let mut cinza_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut ln_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut log10_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut log2_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    for pixel in img.pixels() {
        let valor_intensidade_base = (pixel.2[0] as f32 + pixel.2[1] as f32 + pixel.2[2] as f32) / 3.0;
        let intensidade_base = valor_intensidade_base as u8;
        let intensidade_ln_base = ganho * (1 + intensidade_base).ln() as u8;
        let intensidade_log10_base = ganho * (1 + intensidade_base).log10() as u8;
        let intensidade_log2_base = ganho * (1 + intensidade_base).log(2.0) as u8;

        let valor_intensidade_olho = pixel.2[0] as f32 * 0.2126 + pixel.2[1] as f32 * 0.7152 + pixel.2[2] as f32 * 0.0722;
        let intensidade_olho = valor_intensidade_olho as u8;
        let intensidade_ln_olho = ganho * (1 + intensidade_olho).ln() as u8;
        let intensidade_log10_olho = ganho * (1 + intensidade_olho).log10() as u8;
        let intensidade_log2_olho = ganho * (1 + intensidade_olho).log(2.0) as u8;

        let temp_cinzabase = cinza_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_cinzabase = Rgb([intensidade_base, intensidade_base, intensidade_base]);
        let temp_lnbase = ln_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_lnbase = Rgb([intensidade_ln_base, intensidade_ln_base, intensidade_ln_base]);
        let temp_log10base = log10_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_log10base = Rgb([intensidade_log10_base, intensidade_log10_base, intensidade_log10_base]);
        let temp_log2base = log2_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_log2base = Rgb([intensidade_log2_base, intensidade_log2_base, intensidade_log2_base]);

        let temp_cinzaolho = cinza_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_cinzaolho = Rgb([intensidade_olho, intensidade_olho, intensidade_olho]);
        let temp_lnolho = ln_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_lnolho = Rgb([intensidade_ln_olho, intensidade_ln_olho, intensidade_ln_olho]);
        let temp_log10olho = log10_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_log10olho = Rgb([intensidade_log10_olho, intensidade_log10_olho, intensidade_log10_olho]);
        let temp_log2olho = log2_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_log2olho = Rgb([intensidade_log2_olho, intensidade_log2_olho, intensidade_log2_olho]);
    }

    vec.push((cinza_base.into(), "Imagem_Escala_De_Cinza_Simples".to_string()));
    vec.push((ln_base.into(), "Transformacao_Ln_Simples".to_string()));
    vec.push((log10_base.into(), "Transformacao_Log10_Simples".to_string()));
    vec.push((log2_base.into(), "Transformacao_Log2_Simples".to_string()));

    vec.push((cinza_olho.into(), "Imagem_Escala_De_Cinza_Adaptada".to_string()));
    vec.push((ln_olho.into(), "Transformacao_Ln_Adaptada".to_string()));
    vec.push((log10_olho.into(), "Transformacao_Log10_Adaptada".to_string()));
    vec.push((log2_olho.into(), "Transformacao_Log2_Adaptada".to_string()));

    vec
}

pub fn transformacao_potencia(img: image::DynamicImage, gama: f32) -> Vec<(DynamicImage, String)> {
    let mut vec: Vec<(DynamicImage, String)> = Vec::new();
    let mut cinza_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut cinza_gama_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut potencia_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut potencia_gama_base: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());

    let mut cinza_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut cinza_gama_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut potencia_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    let mut potencia_gama_olho: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(img.width(), img.height());
    for pixel in img.pixels() {
        let valor_intensidade_base = (pixel.2[0] as f32 + pixel.2[1] as f32 + pixel.2[2] as f32) / 3.0;
        let intensidade_base = valor_intensidade_base as u8;
        let intensidade_gama_base = intensidade_base.powf(1.0/gama) as u8;
        let intensidade_potencia_base = if intensidade_base.powf(gama) > 255.0 {255} else {intensidade_base.powf(gama) as u8};
        let intensidade_potencia_gama_base = if intensidade_gama_base.powf(gama) > 255.0 {255} else{intensidade_gama_base.powf(gama) as u8};

        let valor_intensidade_olho = pixel.2[0] as f32 * 0.2126 + pixel.2[1] as f32 * 0.7152 + pixel.2[2] as f32 * 0.0722;
        let intensidade_olho = valor_intensidade_olho as u8;
        let intensidade_gama_olho = intensidade_olho.powf(1.0/gama) as u8;
        let intensidade_potencia_olho = if intensidade_olho.powf(gama) > 255.0 {255} else {intensidade_olho.powf(gama) as u8};
        let intensidade_potencia_gama_olho = if intensidade_gama_olho.log(2.0) > 255.0 {255} else {intensidade_gama_olho.log(2.0) as u8};

        let temp_cinzabase = cinza_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_cinzabase = Rgb([intensidade_base, intensidade_base, intensidade_base]);
        let temp_cinzagamabase = cinza_gama_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_cinzagamabase = Rgb([intensidade_gama_base, intensidade_gama_base, intensidade_gama_base]);
        let temp_potenciabase = potencia_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_potenciabase = Rgb([intensidade_potencia_base, intensidade_potencia_base, intensidade_potencia_base]);
        let temp_potenciagamabase = potencia_gama_base.get_pixel_mut(pixel.0, pixel.1);
        *temp_potenciagamabase = Rgb([intensidade_potencia_gama_base, intensidade_potencia_gama_base, intensidade_potencia_gama_base]);

        let temp_cinzaolho = cinza_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_cinzaolho = Rgb([intensidade_olho, intensidade_olho, intensidade_olho]);
        let temp_cinzagamaolho = cinza_gama_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_cinzagamaolho = Rgb([intensidade_gama_olho, intensidade_gama_olho, intensidade_gama_olho]);
        let temp_potenciaolho = potencia_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_potenciaolho = Rgb([intensidade_potencia_olho, intensidade_potencia_olho, intensidade_potencia_olho]);
        let temp_potenciagamaolho = potencia_gama_olho.get_pixel_mut(pixel.0, pixel.1);
        *temp_potenciagamaolho = Rgb([intensidade_potencia_gama_olho, intensidade_potencia_gama_olho, intensidade_potencia_gama_olho]);
    }

    vec.push((cinza_base.into(), "Imagem_Escala_De_Cinza_Simples".to_string()));
    vec.push((cinza_gama_base.into(), "Imagem_Escala_De_Cinza_Correcao_Gama_Simples".to_string()));
    vec.push((potencia_base.into(), "Transformacao_Potencia_Simples".to_string()));
    vec.push((potencia_gama_base.into(), "Transformacao_Potencia_Correcao_Gama_Simples".to_string()));

    vec.push((cinza_olho.into(), "Imagem_Escala_De_Cinza_Adaptada".to_string()));
    vec.push((cinza_gama_olho.into(), "Imagem_Escala_De_Cinza_Correcao_Gama_Adaptada".to_string()));
    vec.push((potencia_olho.into(), "Transformacao_Potencia_Adaptada".to_string()));
    vec.push((potencia_gama_olho.into(), "Transformacao_Potencia_Correcao_Gama_Adaptada".to_string()));

    vec
}

pub fn salva_transformacao_potencia(
    imgs: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    dir: String,
    name: String,
) -> Result<(), Box<dyn Error>> {
    imgs[0].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaSimples.png"
    ))?;
    imgs[1].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaCorrecaoGamaSimples.png"
    ))?;
    imgs[2].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoPotenciaSimples.png"
    ))?;
    imgs[3].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoPotenciaCorrecaoGamaSimples.png"
    ))?;
    imgs[4].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaAdaptada.png"
    ))?;
    imgs[5].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaCorrecaoGamaAdaptada.png"
    ))?;
    imgs[6].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoPotenciaAdaptada.png"
    ))?;
    imgs[7].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoPotenciaCorrecaoGamaAdaptada.png"
    ))?;
    Ok(())
}

pub fn salva_transformacao_log(
    imgs: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    dir: String,
    name: String,
) -> Result<(), Box<dyn Error>> {
    imgs[0].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaSimples.png"
    ))?;
    imgs[1].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoLnSimples.png"
    ))?;
    imgs[2].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoLog10Simples.png"
    ))?;
    imgs[3].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoLog2Simples.png"
    ))?;
    imgs[4].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaAdaptada.png"
    ))?;
    imgs[5].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoLnAdaptada.png"
    ))?;
    imgs[6].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoLog10Adaptada.png"
    ))?;
    imgs[7].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "TransformacaoLog2Adaptada.png"
    ))?;
    Ok(())
}

pub fn salva_limiarizacao(
    imgs: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    dir: String,
    name: String,
) -> Result<(), Box<dyn Error>> {
    imgs[0].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaSimples.png"
    ))?;
    imgs[1].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "LimiarizacaoSimples.png"
    ))?;
    imgs[2].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "EscalaDeCinzaAdaptada.png"
    ))?;
    imgs[3].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "LimiarizacaoAdaptada.png"
    ))?;
    Ok(())
}

pub fn salva_decomposicao_hsv(
    imgs: Vec<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    dir: String,
    name: String,
) -> Result<(), Box<dyn Error>> {
    imgs[0].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "H.png"
    ))?;
    imgs[1].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "S.png"
    ))?;
    imgs[2].save(format!(
        "{}{}{}",
        dir.trim_end_matches("/").to_owned() + "/",
        name,
        "V.png"
    ))?;
    Ok(())
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
