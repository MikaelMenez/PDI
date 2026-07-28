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
pub fn transformacao_de_intensidade_de_potencia(img: DynamicImage, gama: f32, ganho: f32, ) -> Vec<(DynamicImage, String)> {
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

pub fn equalizacao_histograma(img: DynamicImage, ganho: f32) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let img_gray = img.to_luma8();

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut histograma = [0u32; 256];

    //Preenche histograma
    for pixel in img_gray.pixels() {
        histograma[pixel[0] as usize] += 1;
    }

    //Calcula histograma acumulado
    let mut acumulado = [0u32; 256];
    acumulado[0] = histograma[0];

    for i in 1..256 {
        acumulado[i] = acumulado[i-1] + histograma[i];
    }

    //Calcula a função de mapeamento
    let total_pixels = (width * height) as f32;
    let mut mapeamento = [0u8; 256];

    for i in 0..256 {
        mapeamento[i] = ((acumulado[i] as f32 / total_pixels) * 255.0 * ganho).round() as u8;
    }

    //Mapeamento do histograma
    for (x, y, pixel) in img_gray.enumerate_pixels() {
        let novo_valor = mapeamento[pixel[0] as usize];
        saida.put_pixel(x, y, Rgb([novo_valor, novo_valor, novo_valor]));
    }

    vec.push((DynamicImage::ImageRgb8(saida), "Equalização de Histograma".to_string()));

    vec
}

pub fn fatiamento_intensidade(img: DynamicImage, fLow: u8, fHigh: u8, fundo: bool) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    for pixel in img.pixels() {
        let (x, y) = (pixel.0, pixel.1);
        let rgb = pixel.2;

        let cinza = ((rgb[0] as f32 + rgb[1] as f32 + rgb[2] as f32) / 3.0) as u8;
        let cinza_limiarizado = if cinza >= fLow && cinza <= fHigh { 255 } else { if fundo { cinza } else { 0 } };

        *saida.get_pixel_mut(x, y) = Rgb([cinza_limiarizado, cinza_limiarizado, cinza_limiarizado]);
    }

    vec.push((DynamicImage::ImageRgb8(saida), "Fatiamento por intensidade".to_string()));

    vec
}

pub fn media_gaussiana(img: DynamicImage, sigma: f32) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);

    // ADICIONADO: Converter para escala de cinza
    let img_gray = img.to_luma8();
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    let mut nucleo = vec![vec![0.0f32; 3]; 3];
    let mut soma_nucleo = 0.0f32;
    let raio = (3 / 2);

    // Distribuição Gaussiana no nucleo
    for i in 0..3 {
        for j in 0..3 {
            let x = (i as i32 - raio) as f32;
            let y = (j as i32 - raio) as f32;

            // G(x,y) = (1/(2*PI*sigma^2)) * exp(-(x^2 + y^2)/(2*sigma^2))
            nucleo[i][j] = (1.0 / (2.0 * std::f32::consts::PI * sigma * sigma)) * (-(x*x + y*y) / (2.0 * sigma * sigma)).exp();

            soma_nucleo += nucleo[i][j];
        }
    }

    // Normalização do nucleo
    for i in 0..3 {
        for j in 0..3 {
            nucleo[i][j] /= soma_nucleo;
        }
    }

    for y in 0..height {
        for x in 0..width {
            let mut soma = 0.0f32;

            // Percorrer a vizinhança do pixel
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = x as i32 + (kx as i32 - raio);
                    let py = y as i32 + (ky as i32 - raio);

                    // Tratamento de bordas (espelhamento)
                    let px_clamp = if px < 0 { -px }
                    else if px >= width as i32 { 2 * (width as i32) - px - 2 }
                    else { px };
                    let py_clamp = if py < 0 { -py }
                    else if py >= height as i32 { 2 * (height as i32) - py - 2 }
                    else { py };

                    let pixel = img_gray.get_pixel(px_clamp as u32, py_clamp as u32);
                    let peso = nucleo[ky][kx];

                    soma += pixel[0] as f32 * peso;
                }
            }

            saida.put_pixel(x, y, Rgb([soma.round() as u8, soma.round() as u8, soma.round() as u8]));
        }
    }

    vec.push((DynamicImage::ImageRgb8(saida), "Filtro de media gaussiana".to_string()));

    vec
}

pub fn filtro_agucamento(img: DynamicImage, ganho: f32) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let img_gray = img.to_luma8();

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    //Nucleo de ganho ajustável
    let nucleo = [
        [0.0, -1.0,  0.0],
        [-1.0, 5.0, -1.0],
        [0.0, -1.0,  0.0]
    ];

    for y in 0..height {
        for x in 0..width {
            let mut soma = 0.0f32;

            //Percorrer a vizinhança do pixel
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = x as i32 + (kx as i32 - 1);
                    let py = y as i32 + (ky as i32 - 1);

                    //Tratamento de bordas (espelhamento)
                    let px_clamp = if px < 0 { -px }
                    else if px >= width as i32 { 2 * (width as i32) - px - 2 }
                    else { px };
                    let py_clamp = if py < 0 { -py }
                    else if py >= height as i32 { 2 * (height as i32) - py - 2 }
                    else { py };

                    let pixel = img_gray.get_pixel(px_clamp as u32, py_clamp as u32);
                    let peso = nucleo[ky][kx];

                    soma += pixel[0] as f32 * peso;
                }
            }

            let pixel = img_gray.get_pixel(x, y);
            let novo = (pixel[0] as f32 + ganho * soma).clamp(0.0, 255.0) as u8;

            saida.put_pixel(x, y, Rgb([novo, novo, novo]));

        }
    }

    vec.push((DynamicImage::ImageRgb8(saida), "Agucamento com ganho ajustavel".to_string()));

    vec
    }

pub fn agucamento_laplaciano(img: DynamicImage, ganho: f32) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let img_gray = img.to_luma8();

    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);
    let mut saida_4v: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut filtro_4v: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut saida_8v: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut filtro_8v: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    //Nucleo de ganho ajustável
    let nucleo_4v: [[f32; 3]; 3] = [
        [0.0, -1.0,  0.0],
        [-1.0, 4.0, -1.0],
        [0.0, -1.0,  0.0]
    ];
    let nucleo_8v: [[f32; 3]; 3] = [
        [-1.0, -1.0, -1.0],
        [-1.0,  8.0, -1.0],
        [-1.0, -1.0, -1.0]
    ];

    for y in 0..height {
        for x in 0..width {
            let mut soma_4v = 0.0f32;

            let mut soma_8v = 0.0f32;

            //Percorrer a vizinhança do pixel
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = x as i32 + (kx as i32 - 1);
                    let py = y as i32 + (ky as i32 - 1);

                    //Tratamento de bordas (espelhamento)
                    let px_clamp = if px < 0 { -px }
                    else if px >= width as i32 { 2 * (width as i32) - px - 2 }
                    else { px };
                    let py_clamp = if py < 0 { -py }
                    else if py >= height as i32 { 2 * (height as i32) - py - 2 }
                    else { py };

                    let pixel = img_gray.get_pixel(px_clamp as u32, py_clamp as u32);
                    let peso_4v = nucleo_4v[ky][kx];
                    let peso_8v = nucleo_8v[ky][kx];

                    soma_4v += pixel[0] as f32 * peso_4v;

                    soma_8v += pixel[0] as f32 * peso_8v;
                }
            }

            let pixel = img.get_pixel(x, y);

            filtro_4v.put_pixel(x, y, Rgb([soma_4v.clamp(0.0, 255.0) as u8, soma_4v.clamp(0.0, 255.0) as u8, soma_4v.clamp(0.0, 255.0) as u8]));

            let novo_4v = (pixel[0] as f32 + ganho * soma_4v).clamp(0.0, 255.0) as u8;
            saida_4v.put_pixel(x, y, Rgb([novo_4v, novo_4v, novo_4v]));

            filtro_8v.put_pixel(x, y, Rgb([soma_8v.clamp(0.0, 255.0) as u8, soma_8v.clamp(0.0, 255.0) as u8, soma_8v.clamp(0.0, 255.0) as u8]));

            let novo_8v = (pixel[0] as f32 + ganho * soma_8v).clamp(0.0, 255.0) as u8;
            saida_8v.put_pixel(x, y, Rgb([novo_8v, novo_8v, novo_8v]));

        }
    }

    vec.push((DynamicImage::ImageRgb8(filtro_4v), "Filtro Laplaciano de 4vizinhancas".to_string()));
    vec.push((DynamicImage::ImageRgb8(saida_4v), "Agucamento Laplaciano de 4vizinhancas".to_string()));

    vec.push((DynamicImage::ImageRgb8(filtro_8v), "Filtro Laplaciano de 8vizinhancas".to_string()));
    vec.push((DynamicImage::ImageRgb8(saida_8v), "Agucamento Laplaciano de 8vizinhancas".to_string()));

    vec
}

pub fn agucamento_sobel(img: DynamicImage, fator: f32) -> Vec<(DynamicImage, String)> {
    let (width, height) = img.dimensions();
    let mut vec: Vec<(DynamicImage, String)> = Vec::with_capacity(1);

    let img_gray = img.to_luma8();
    let mut saida: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let mut filtro: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // Kernels de Sobel
    let sobel_x = [
        [-1.0, 0.0, 1.0],
        [-2.0, 0.0, 2.0],
        [-1.0, 0.0, 1.0]
    ];

    let sobel_y = [
        [-1.0, -2.0, -1.0],
        [0.0,  0.0,  0.0],
        [1.0,  2.0,  1.0]
    ];

    let mut gradiente = vec![vec![0.0f32; width as usize]; height as usize];
    let mut max_gradiente = 0.0f32;

    for y in 1..height-1 {
        for x in 1..width-1 {
            let mut gx = 0.0f32;
            let mut gy = 0.0f32;

            // Aplica os nucleos de Sobel
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = x as i32 + (kx as i32 - 1);
                    let py = y as i32 + (ky as i32 - 1);

                    let pixel = img_gray.get_pixel(px as u32, py as u32);
                    let valor = pixel[0] as f32;

                    gx += valor * sobel_x[ky as usize][kx as usize];
                    gy += valor * sobel_y[ky as usize][kx as usize];
                }
            }

            // Magnitude do gradiente
            let magnitude = (gx * gx + gy * gy).sqrt();
            gradiente[y as usize][x as usize] = magnitude;

            if magnitude > max_gradiente {
                max_gradiente = magnitude;
            }
        }
    }

    for y in 0..height {
        for x in 0..width {
            let original = img_gray.get_pixel(x, y)[0] as f32;

            // Normalizar gradiente para [0, 1]
            let grad_norm = if max_gradiente > 0.0 { gradiente[y as usize][x as usize] / max_gradiente } else { 0.0 };

            // Aguçamento: original + fator * gradiente
            let novo_valor = (original + fator * grad_norm * 255.0).clamp(0.0, 255.0) as u8;
            saida.put_pixel(x, y, Rgb([novo_valor, novo_valor, novo_valor]));
            
            filtro.put_pixel(x, y, Rgb([grad_norm as u8, grad_norm as u8, grad_norm as u8]));
        }
        _ => {
            let gray = img.to_luma8();
            let (width, height) = gray.dimensions();
            let mut saida = ImageBuffer::new(width, height);

            for (x, y, pixel) in gray.enumerate_pixels() {
                let r1: f32 = rng.gen_range(0.0..1.0);
                if r1 < probabilidade_total {
                    let r2: f32 = rng.gen_range(0.0..1.0);
                    let val = if r2 < proporcao_sal { 255u8 } else { 0u8 };
                    saida.put_pixel(x, y, Luma([val]));
                } else {
                    saida.put_pixel(x, y, *pixel);
                }
            }
            DynamicImage::ImageLuma8(saida)
        }
    };

    vec.push((DynamicImage::ImageRgb8(saida), "Imagem com aguçamento por gradiente de Sobel".to_string()));
    vec.push((DynamicImage::ImageRgb8(filtro), "Filtro de aguçamento por gradiente de Sobel".to_string()));

    vec
}




