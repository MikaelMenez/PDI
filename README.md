
# Processador de Imagens (Rust + Slint)

Uma aplicação desktop nativa, extremamente leve e multiplataforma para processamento de imagens. O projeto utiliza o framework **Slint UI** para a interface gráfica moderna e a linguagem **Rust** para garantir performance bruta e segurança de memória no processamento dos algoritmos, seguindo uma arquitetura que separa completamente o design da lógica de negócios.

 *Interface inspirada em wireframes de processamento científico.*

---

## 🚀 Funcionalidades

* **Abertura de Arquivos Nativa:** Integração com o sistema operacional via `rfd` para selecionar arquivos `.png`.
* **Lista Dinâmica de Processos:** Sidebar de filtros injetada diretamente pelo backend em Rust.
* **Interface Reativa:** Layout auto-ajustável que se adapta a redimensionamentos sem quebrar os painéis.
* **Zero Tecnologias Web:** Interface nativa compilada estaticamente, sem HTML, CSS ou JavaScript.

---

## 📁 Estrutura do Projeto

Para que a compilação automática funcione, certifique-se de que a estrutura de arquivos do seu diretório está exatamente assim:

```text
meu_app_slint/
├── ui/
│   └── app.slint       # Código declarativo da interface gráfica
├── src/
│   └── main.rs         # Lógica principal e handlers de eventos em Rust
├── build.rs            # Script de compilação automática do Slint
└── Cargo.toml          # Dependências do projeto

```

---

## 🛠️ Pré-requisitos

Antes de começar, você precisará ter o ecossistema Rust instalado na sua máquina:

* **Rust (Cargo & rustc)** - Instale via [rustup.rs](https://rustup.rs/)

*(Nota para usuários de Linux/Kubuntu: O Slint utiliza aceleração por hardware via OpenGL/Vulkan. Geralmente, as bibliotecas padrão do sistema já são suficientes).*

---

## ⚙️ Passo a Passo para Rodar a Aplicação

Siga os passos abaixo no seu terminal para configurar e executar o projeto:

### 1. Configurar as Dependências (`Cargo.toml`)

Garanta que as seguintes dependências estão declaradas no seu arquivo `Cargo.toml`:

```toml
[package]
name = "meu_app_slint"
version = "0.1.0"
edition = "2021"

[dependencies]
slint = "1.9"
rfd = "0.15"

[build-dependencies]
slint-build = "1.9"

```

### 2. Compilar e Rodar o Projeto

Na raiz do projeto (onde está o arquivo `Cargo.toml`), execute o comando padrão do Cargo:

```bash
cargo run

```

> **O que vai acontecer aqui?**
> O Cargo detectará o arquivo `build.rs`, compilará o seu arquivo de interface `app.slint` gerando código Rust fortemente tipado por baixo dos panos e, em seguida, compilará e abrirá a janela do seu aplicativo.

---

## 🎨 Desenvolvimento Ágil: Modificando a Interface Sem Compilar Rust

Se você quiser alterar cores, margens ou a disposição dos blocos no arquivo `app.slint`, você não precisa passar pelo processo de compilação do Rust (`cargo run`) a cada mudança. Use o visualizador em tempo real do Slint:

1. **Instale a ferramenta globalmente:**
```bash
cargo install slint-viewer

```


2. **Rode o preview apontando para o seu arquivo:**
```bash
slint-viewer ui/app.slint

```



A janela abrirá instantaneamente. Cada alteração salva no arquivo `app.slint` será refletida no visualizador na mesma hora, mantendo o seu fluxo de desenvolvimento livre de estresse.
