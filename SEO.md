# Estratégia de SEO & Arquitetura de Busca do Nexa (nexa.github.io)

Este documento descreve detalhadamente a estratégia de **SEO Técnico, On-Page, Semântico e de Arquitetura de Informação** implementada no site oficial do **Nexa**. O objetivo é garantir indexação precisa, autoridade temática e descoberta orgânica sustentável sem práticas de black hat ou spam.

---

## 1. Estratégia Geral de SEO Adotada

### 1.1 Posicionamento da Entidade Semântica (Brand Disambiguation)
Como a palavra *"Nexa"* é compartilhada por criptomoedas, modelos de inteligência artificial e empresas de mineração, a estratégia não busca competir por termos genéricos soltos. Em vez disso, estabelece o Nexa no Knowledge Graph do Google sob a entidade:
- **Entidade**: `Nexa KVM Share` / `Nexa Software`
- **Categoria**: `SoftwareApplication` (DeveloperApplication / Utilities)
- **Associação de Termos**: KVM por software, compartilhamento de periféricos em rede local, Rust 2021, Windows 11, Wayland, ChaCha20-Poly1305.

### 1.2 URLs Limpas (Clean Directory Routing)
Em vez de expor arquivos com extensão `.html` (ex: `site/docs.html`), adotamos subpastas dedicadas com arquivos `index.html` internos:
- `https://nexa.github.io/como-funciona/`
- `https://nexa.github.io/recursos/`
- `https://nexa.github.io/instalacao/`
- `https://nexa.github.io/instalacao/windows/`
- `https://nexa.github.io/instalacao/linux/`
- `https://nexa.github.io/seguranca/`
- `https://nexa.github.io/solucao-de-problemas/`
- `https://nexa.github.io/documentacao/`
- `https://nexa.github.io/faq/`
- `https://nexa.github.io/roadmap/`

Isso melhora o CTR nos resultados de busca e confere estrutura canônica estável no GitHub Pages.

---

## 2. Palavras-Chave Principais (Primary Keywords)

Identificadas a partir da análise direta do código-fonte:
1. `Nexa KVM`
2. `Nexa software`
3. `KVM por software`
4. `compartilhar mouse e teclado na rede`
5. `controlar Linux pelo Windows mouse teclado`
6. `KVM Rust`
7. `alternativa Barrier Deskflow Synergy`
8. `software KVM Wayland`

---

## 3. Palavras-Chave Secundárias & Long-Tail

- **Técnicas / Protocolo**: `ChaCha20-Poly1305 KVM`, `pareamento SAS PIN KVM`, `coalescência mouse gamer 1000Hz`, `framing binário CRC32`.
- **Instalação / Sistema Operacional**: `instalar nexa windows 11`, `permissão /dev/uinput sem root`, `liberar porta 25800 firewall windows`, `nexa debian deb package`, `nexa elementary os 8.1`.
- **Suporte / Troubleshooting**: `nexa connection timeout AP isolation`, `nexa connection refused firewall`, `soltar cursor tela remota nexa esc`.

---

## 4. Intenção de Busca por Página

| Rota | Tipo de Intenção | Pergunta que a Página Responde |
| :--- | :--- | :--- |
| `/` | Navegacional & Apresentação | "O que é o Nexa e como ele me ajuda a controlar dois PCs?" |
| `/como-funciona/` | Informacional & Arquitetural | "Como o Nexa move o cursor sem lag e sincroniza o clipboard?" |
| `/recursos/` | Comparativa & Comercial | "Quais os diferenciais do Nexa contra Barrier, Synergy e KVM físico?" |
| `/instalacao/` | Guia / Hub | "Onde baixo o Nexa e quais são os manuais por sistema operacional?" |
| `/instalacao/windows/` | How-To & Procedural | "Como instalar no Windows 11 e evitar travamento em janelas UAC?" |
| `/instalacao/linux/` | How-To & Procedural | "Como configurar o Nexa no elementary OS 8.1 Wayland e /dev/uinput?" |
| `/seguranca/` | Confiança & Informacional | "É seguro digitar senhas com o Nexa? Como funciona a criptografia?" |
| `/solucao-de-problemas/`| Suporte / Long-tail | "Por que dá erro de Timeout ou Connection Refused e como resolver?" |
| `/documentacao/` | Referência Técnica | "Quais são as flags da CLI e como preencher o arquivo nexa.toml?" |
| `/faq/` | Resposta Rápida (FAQ) | "Funciona sem internet? Dá para jogar jogos? Qual a licença?" |
| `/roadmap/` | Transparência & Comunidade | "Qual o estado atual do projeto e o que está planejado?" |

---

## 5. Arquitetura de Links Internos (Internal Linking)

A malha de links segue o conceito de **Hub & Spoke**:
- **Hub Central (Home)**: Apresenta o produto e distribui autoridade (*Link Equity*) para todas as seções principais.
- **Hubs Temáticos**: A página `/instalacao/` direciona especificamente para os spokes `/instalacao/windows/` e `/instalacao/linux/`.
- **Links Contextuais Cruzados**:
  - A página `/como-funciona/` faz link contextual para `/seguranca/`.
  - A página `/instalacao/linux/` aponta para `/solucao-de-problemas/` ao falar de permissões.
  - O rodapé semântico presente em 100% das páginas fornece links estruturados para todas as categorias.

---

## 6. Dados Estruturados Schema.org (JSON-LD) Implementados

Foram aplicados esquemas padronizados e validados:
1. **`WebSite` e `Organization`** (Home): Estabelecem a organização oficial, logotipo e links para redes/GitHub.
2. **`SoftwareApplication`** (Home e Recursos): Declara o sistema operacional, categoria, gratuidade (Free/Open Source) e lista de recursos.
3. **`BreadcrumbList`** (Todas as subpáginas): Garante a exibição da trilha de navegação (ex.: `Início > Instalação > Windows`) nas SERPs do Google.
4. **`HowTo` com `HowToStep`** (Páginas de Instalação): Otimizado para permitir rich cards de passo a passo diretamente na busca do Google.
5. **`TechArticle`** (Como Funciona, Segurança e Documentação): Sinaliza conteúdo técnico de engenharia de software com autoria de entidade.
6. **`FAQPage`** (FAQ e Solução de Problemas): Qualifica as páginas para receber expansores de sanfona interativos no Google.

---

## 7. Sitemap & Robots.txt

- **`sitemap.xml`**: Contém todas as 11 URLs canônicas limpas, com `<priority>` proporcional (1.0 para a Home, 0.9 para Guias de Instalação e 0.8 para documentação e roadmap), além de tags `<lastmod>` atualizadas.
- **`robots.txt`**: Permite o rastreamento irrestrito de todo o conteúdo público e aponta o link absoluto para o sitemap:
  ```text
  User-agent: *
  Allow: /

  Sitemap: https://nexa.github.io/sitemap.xml
  ```

---

## 8. Estratégia de Conteúdo e E-E-A-T

O conteúdo reflete com fidelidade os arquivos fontes de Rust do projeto:
- Nomes exatos de variáveis e crates (ex: `nexa-protocol`, `nexa-crypto`, `nexa-platform`).
- Constantes reais do código: porta TCP padrão `25800`, porta de broadcast UDP `25801`, porta da GUI `25802`.
- Transparência total: explicitado que o modo Host no Wayland está em desenvolvimento ativo, enquanto o modo Guest via `/dev/uinput` está 100% funcional com testes automatizados passando.

---

## 9. Estratégia Internacional

O site inicial foi construído em português do Brasil (`pt-BR`) por ser o idioma nativo da documentação do desenvolvedor, com tags de controle de idioma:
- `<html lang="pt-BR">`
- `<link rel="alternate" hreflang="pt-BR" href="https://nexa.github.io/...">`
- `<link rel="alternate" hreflang="x-default" href="https://nexa.github.io/...">`

Para o futuro suporte em inglês (`/en/`), a arquitetura de pastas está pronta para espelhar a mesma árvore com `hreflang="en"`.

---

## 10. Passos Recomendados Pós-Publicação

1. **Google Search Console**:
   - Cadastre a propriedade `https://nexa.github.io/` (ou `https://rafaelmaciels.github.io/Nexa/`).
   - Insira o token de verificação HTML na linha reservada em `index.html`:
     ```html
     <!-- <meta name="google-site-verification" content="SEU_TOKEN_AQUI" /> -->
     ```
   - Envie o endereço do sitemap: `https://nexa.github.io/sitemap.xml`.
2. **Bing Webmaster Tools**:
   - Importe a propriedade verificada do Google Search Console com 1 clique para garantir presença no ecossistema Bing e DuckDuckGo.
3. **Link Building Natural**:
   - Atualize a URL do site no campo "Website" do repositório no GitHub (`https://github.com/rafaelmaciels/Nexa`).
   - Referencie o site nos manuais do repositório (`README.md`, `MANUAL_WINDOWS.md`, `MANUAL_LINUX.md`).
