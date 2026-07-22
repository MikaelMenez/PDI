use image::DynamicImage;
use slint::{ComponentHandle, Image, ModelRc, SharedPixelBuffer, VecModel};
use std::cell::RefCell;
use std::rc::Rc;

mod processos;

slint::include_modules!();

const COR_SUCESSO: slint::Color = slint::Color::from_rgb_u8(166, 227, 161);
const COR_ERRO: slint::Color = slint::Color::from_rgb_u8(243, 139, 168);
const COR_INFO: slint::Color = slint::Color::from_rgb_u8(138, 173, 244);

/// Todos os parâmetros configuráveis na interface, já lidos e tipados.
/// Toda função de processamento recebe uma referência a isto — use só os
/// campos que fizerem sentido pro seu processo e ignore o resto.
pub struct Parametros {
    pub param_1: f32,
    pub param_2: f32,
    pub kernel: u32,
    pub sigma: f32,
    pub freq_corte: f32,
    pub ordem: u32,
    pub faixa_a: f32,
    pub faixa_b: f32,
    pub preservar_fundo: bool,
    pub intensidade_ruido: f32,
    pub distribuicao_ruido: f32,
}

fn extrair_parametros(app: &AppWindow) -> Parametros {
    Parametros {
        param_1: app.get_param_1(),
        param_2: app.get_param_2(),
        kernel: app.get_param_kernel().max(1) as u32,
        sigma: app.get_desvio_padrao_sigma(),
        freq_corte: app.get_freq_corte(),
        ordem: app.get_ordem_filtro().max(1) as u32,
        faixa_a: app.get_faixa_a(),
        faixa_b: app.get_faixa_b(),
        preservar_fundo: app.get_preservar_fundo(),
        intensidade_ruido: app.get_intensidade_ruido(),
        distribuicao_ruido: app.get_distribuicao_ruido(),
    }
}

/// Converte uma DynamicImage (crate `image`) para slint::Image.
fn dynimg_para_slint(img: &DynamicImage) -> Image {
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Image::from_rgba8(SharedPixelBuffer::clone_from_slice(rgba.as_raw(), w, h))
}

/// ---------------------------------------------------------------------------
/// TABELA DE PROCESSOS — o único lugar que muda quando você implementa algo novo
/// ---------------------------------------------------------------------------
/// Passo a passo pra adicionar um processo:
///   1. Escreva em processos.rs uma função com esta assinatura:
///        pub fn minha_funcao(img: DynamicImage, p: &Parametros) -> Vec<(DynamicImage, String)>
///      (pode ignorar os campos de `p` que não usar)
///   2. Acrescente UMA linha abaixo, com o nome EXATO usado na lista
///      `filtros` do arquivo .slint.
/// Conversão pra tela, galeria de resultados, imagem principal e status são
/// tratados automaticamente — não precisa tocar em mais nada.
fn executar_processo(
    nome: &str,
    img: DynamicImage,
    p: &Parametros,
) -> Option<Vec<(DynamicImage, String)>> {
    match nome {
        "Decomposição RGB" => Some(processos::decomposicao_rgb(img)),
        "Decomposição HSV" => Some(processos::decomposicao_hsv(img)),
        "Limiarização" => Some(processos::limiarizacao(
            img,
            p.param_1.clamp(0.0, 255.0) as u8,
        )),
        "Transf. Logarítmica" => Some(processos::transformacao_log(img, p.param_1)),
        "Transformação Potência" => Some(processos::transformacao_potencia(img, p.param_2)),

        // Vá acrescentando aqui à medida que implementar em processos.rs, ex:
        // "Equalização de Histograma" => Some(processos::equalizacao_histograma(img)),
        // "Fatiamento por Intensidade" => Some(processos::fatiamento_intensidade(img, p)),
        // "Filtro de Média Gaussiana" => Some(processos::filtro_media_gaussiana(img, p)),
        // "Filtro de Mediana" => Some(processos::filtro_mediana(img, p)),
        // "Filtro de Mínimo" => Some(processos::filtro_minimo(img, p)),
        // "Filtro de Máximo" => Some(processos::filtro_maximo(img, p)),
        // "Máscara de Aguçamento" => Some(processos::mascara_de_agucamento(img, p)),
        // "Realce por Laplaciano" => Some(processos::realce_laplaciano(img)),
        // "Gradiente de Sobel" => Some(processos::gradiente_sobel(img)),
        // "Passa-Baixa Gaussiano" => Some(processos::passa_baixa_gaussiano(img, p)),
        // "Passa-Alta Gaussiano" => Some(processos::passa_alta_gaussiano(img, p)),
        // "Passa-Baixa Butterworth" => Some(processos::passa_baixa_butterworth(img, p)),
        // "Passa-Alta Butterworth" => Some(processos::passa_alta_butterworth(img, p)),
        // "Filtro Adaptativo de Mediana" => Some(processos::filtro_adaptativo_mediana(img, p)),
        // "Ruído Aditivo Gaussiano" => Some(processos::ruido_aditivo_gaussiano(img, p)),
        // "Ruído Sal, Pimenta, Sal e Pimenta" => Some(processos::ruido_sal_pimenta(img, p)),
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = AppWindow::new()?;

    let lista_saidas_model: Rc<VecModel<ImagemSaida>> = Rc::new(VecModel::default());
    app.set_lista_saidas(ModelRc::from(lista_saidas_model.clone()));

    // Imagem original carregada — mantida fora do Slint pra poder reprocessar
    // sempre que o usuário mudar parâmetros e apertar "Executar" de novo.
    let imagem_entrada: Rc<RefCell<Option<DynamicImage>>> = Rc::new(RefCell::new(None));

    // Últimos resultados (DynamicImage + nome), na mesma ordem da galeria —
    // é a partir daqui que "Salvar" e "Salvar Todas" leem os dados de fato.
    let resultados_atuais: Rc<RefCell<Vec<(DynamicImage, String)>>> =
        Rc::new(RefCell::new(Vec::new()));

    // ------------------------------------------------------------------------
    // 1. ABRIR ARQUIVO
    // ------------------------------------------------------------------------
    let app_weak = app.as_weak();
    let imagem_entrada_abrir = imagem_entrada.clone();
    app.on_abrir_arquivo(move || {
        let app = match app_weak.upgrade() {
            Some(a) => a,
            None => return,
        };

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Imagens PNG", &["png"])
            .pick_file()
        {
            match image::open(&path) {
                Ok(dyn_img) => {
                    app.set_img_entrada(dynimg_para_slint(&dyn_img));
                    app.set_img_saida(Image::default());
                    app.set_indice_saida_selecionada(-1);
                    *imagem_entrada_abrir.borrow_mut() = Some(dyn_img);

                    app.set_status_texto(
                        format!(
                            "Imagem carregada: {:?}",
                            path.file_name().unwrap_or_default()
                        )
                        .into(),
                    );
                    app.set_status_cor(COR_SUCESSO);
                }
                Err(e) => {
                    app.set_status_texto(format!("Erro ao carregar imagem: {}", e).into());
                    app.set_status_cor(COR_ERRO);
                }
            }
        }
    });

    // ------------------------------------------------------------------------
    // 2. EXECUTAR PROCESSO
    // ------------------------------------------------------------------------
    let app_weak = app.as_weak();
    let imagem_entrada_proc = imagem_entrada.clone();
    let resultados_proc = resultados_atuais.clone();
    let lista_saidas_model_proc = lista_saidas_model.clone();
    app.on_processar(move || {
        let app = match app_weak.upgrade() {
            Some(a) => a,
            None => return,
        };

        let img = match imagem_entrada_proc.borrow().clone() {
            Some(img) => img,
            None => {
                app.set_status_texto("Carregue uma imagem antes de processar.".into());
                app.set_status_cor(COR_ERRO);
                return;
            }
        };

        let nome_processo = app.get_nome_processo();
        let params = extrair_parametros(&app);

        match executar_processo(nome_processo.as_str(), img, &params) {
            Some(resultados) => {
                if resultados.is_empty() {
                    app.set_status_texto("Processo não retornou nenhuma imagem.".into());
                    app.set_status_cor(COR_ERRO);
                    return;
                }

                // Popula a galeria de miniaturas
                let itens: Vec<ImagemSaida> = resultados
                    .iter()
                    .map(|(img, nome)| ImagemSaida {
                        nome: nome.clone().into(),
                        imagem: dynimg_para_slint(img),
                    })
                    .collect();
                lista_saidas_model_proc.set_vec(itens);

                // Mostra a primeira imagem como resultado principal
                app.set_img_saida(dynimg_para_slint(&resultados[0].0));
                app.set_indice_saida_selecionada(0);

                app.set_status_texto(
                    format!(
                        "'{}' aplicado com sucesso! {} imagem(ns) gerada(s).",
                        nome_processo,
                        resultados.len()
                    )
                    .into(),
                );
                app.set_status_cor(COR_SUCESSO);

                *resultados_proc.borrow_mut() = resultados;
            }
            None => {
                app.set_status_texto(
                    format!(
                        "'{}' ainda não foi implementado em processos.rs.",
                        nome_processo
                    )
                    .into(),
                );
                app.set_status_cor(COR_INFO);
            }
        }
    });

    // ------------------------------------------------------------------------
    // 3. SELECIONAR UMA SAÍDA DA GALERIA (mostra em tamanho grande)
    // ------------------------------------------------------------------------
    let app_weak = app.as_weak();
    let resultados_selecionar = resultados_atuais.clone();
    app.on_selecionar_saida(move |indice| {
        let app = match app_weak.upgrade() {
            Some(a) => a,
            None => return,
        };
        let resultados = resultados_selecionar.borrow();
        if let Some((img, _nome)) = resultados.get(indice as usize) {
            app.set_img_saida(dynimg_para_slint(img));
            app.set_indice_saida_selecionada(indice);
        }
    });

    // ------------------------------------------------------------------------
    // 4. SALVAR SAÍDA SELECIONADA
    // ------------------------------------------------------------------------
    let app_weak = app.as_weak();
    let resultados_salvar = resultados_atuais.clone();
    app.on_salvar_atual(move || {
        let app = match app_weak.upgrade() {
            Some(a) => a,
            None => return,
        };

        let indice = app.get_indice_saida_selecionada();
        let resultados = resultados_salvar.borrow();

        let imagem = if indice >= 0 {
            resultados.get(indice as usize).map(|(img, _)| img)
        } else {
            resultados.first().map(|(img, _)| img)
        };

        let imagem = match imagem {
            Some(img) => img,
            None => {
                app.set_status_texto("Nenhum resultado disponível para salvar.".into());
                app.set_status_cor(COR_ERRO);
                return;
            }
        };

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_file_name("resultado.png")
            .save_file()
        {
            match imagem.save(&path) {
                Ok(_) => {
                    app.set_status_texto(format!("Imagem salva em {:?}", path).into());
                    app.set_status_cor(COR_SUCESSO);
                }
                Err(e) => {
                    app.set_status_texto(format!("Erro ao salvar: {}", e).into());
                    app.set_status_cor(COR_ERRO);
                }
            }
        }
    });

    // ------------------------------------------------------------------------
    // 5. SALVAR TODAS AS SAÍDAS (.zip)
    // ------------------------------------------------------------------------
    let app_weak = app.as_weak();
    let resultados_zip = resultados_atuais.clone();
    app.on_salvar_todas(move || {
        let app = match app_weak.upgrade() {
            Some(a) => a,
            None => return,
        };

        let resultados = resultados_zip.borrow();
        if resultados.is_empty() {
            app.set_status_texto("Nenhum resultado disponível para exportar.".into());
            app.set_status_cor(COR_ERRO);
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ZIP", &["zip"])
            .set_file_name("resultados.zip")
            .save_file()
        {
            match salvar_resultados_zip(&resultados, &path) {
                Ok(_) => {
                    app.set_status_texto(
                        format!("{} imagem(ns) exportada(s) em {:?}", resultados.len(), path)
                            .into(),
                    );
                    app.set_status_cor(COR_SUCESSO);
                }
                Err(e) => {
                    app.set_status_texto(format!("Erro ao exportar ZIP: {}", e).into());
                    app.set_status_cor(COR_ERRO);
                }
            }
        }
    });

    app.run()?;
    Ok(())
}

/// Compacta todos os resultados atuais num único .zip.
/// Requer o crate `zip` no Cargo.toml (veja instruções na conversa).
fn salvar_resultados_zip(
    resultados: &[(DynamicImage, String)],
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use zip::write::FileOptions;

    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options: FileOptions<()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for (i, (img, nome)) in resultados.iter().enumerate() {
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png)?;

        let nome_arquivo = format!("{:02}_{}.png", i + 1, nome);
        zip.start_file(nome_arquivo, options)?;
        zip.write_all(buffer.get_ref())?;
    }

    zip.finish()?;
    Ok(())
}
