//! Optional settings page and page help in Purple-lib (its `API.md`). Purple-lib calls the plugin's
//! `purplelib_on_register` with its table of functions; without Purple-lib nothing here runs. The
//! records and the table mirror Purple-lib's API version 3 (Purple-lib r17 and later); a later API
//! version still reads them. Row texts and the mapping between row values and `Settings` live here
//! so the host tests can check them.
use crate::{Preview, Settings, Style};
use core::ffi::{CStr, c_char, c_void};

pub const API_VERSION: u32 = 3;

pub const ROW_LABELS: u32 = 0;
pub const ROW_NUMBERS: u32 = 1;
pub const ROW_TEXT_KIND: u32 = 2;
pub const ROW_ACTION: u32 = 3;
pub const ROW_NO_WRAP: u32 = 1;

#[repr(C)]
/// Purple-lib zeroes it before each call.
pub struct OptionRow {
    pub name: *const c_char,
    pub help: *const c_char,
    pub labels: *const *const c_char,
    pub label_count: u32,
    pub kind: u32,
    pub value_count: u32,
    pub first: i32,
    pub step: i32,
    pub suffix: *const c_char,
    pub flags: u32,
}

#[repr(C)]
pub struct Options {
    pub size: u32,
    pub context: *mut c_void,
    pub title: Option<extern "C" fn(*mut c_void, u32) -> *const c_char>,
    pub row_count: u32,
    pub row: Option<extern "C" fn(*mut c_void, u32, u32, *mut OptionRow) -> bool>,
    pub get: Option<extern "C" fn(*mut c_void, u32) -> u32>,
    pub set: Option<extern "C" fn(*mut c_void, u32, u32)>,
    pub closed: Option<extern "C" fn(*mut c_void)>,
    pub value_text: Option<extern "C" fn(*mut c_void, u32, u32, u32) -> *const c_char>,
    pub action: Option<extern "C" fn(*mut c_void, u32)>,
}

#[repr(C)]
pub struct Help {
    pub size: u32,
    pub context: *mut c_void,
    pub category: u32,
    pub page_count: u32,
    pub title: Option<extern "C" fn(*mut c_void, u32) -> *const c_char>,
    pub page: Option<extern "C" fn(*mut c_void, u32, u32) -> *const c_char>,
    pub open: Option<extern "C" fn(*mut c_void)>,
}

/// Purple-lib's functions, as it passes them to `purplelib_on_register`; valid for the whole run.
#[repr(C)]
pub struct Api {
    pub version: u32,
    pub add_options: extern "C" fn(*const Options) -> bool,
    pub add_help: extern "C" fn(*const Help) -> bool,
    pub hold_input: extern "C" fn(),
    pub options_shown: extern "C" fn(*const Options) -> bool,
    pub options_row: extern "C" fn(*const Options) -> i32,
    pub add_set_help: extern "C" fn(*const Options, *const Help) -> bool,
}

#[cfg(target_pointer_width = "32")]
const _: () = assert!(
    core::mem::size_of::<Options>() == 40
        && core::mem::size_of::<Help>() == 28
        && core::mem::size_of::<OptionRow>() == 40
        && core::mem::size_of::<Api>() == 28
);

pub const TITLE: &CStr = c"Damage Numbers";
pub const ROW_ENABLED: u32 = 0;
pub const ROW_STYLE: u32 = 1;
pub const ROW_SIZE: u32 = 2;
pub const ROW_DURATION: u32 = 3;
pub const ROW_SPREAD: u32 = 4;
pub const ROW_PREVIEW: u32 = 5;
pub const ROW_RANDOM: u32 = 6;
pub const ROW_RESTORE: u32 = 7;
pub const ROWS: u32 = 8;

/// English, French, Spanish, German, Italian, Traditional and Simplified Chinese.
pub const LANGUAGES: usize = 7;
/// Index into the tables from the game's language byte, which Purple-lib passes (anything unknown
/// is English).
pub const fn language_index(byte: u32) -> usize {
    match byte {
        2 => 1,
        3 => 2,
        4 => 3,
        5 => 4,
        7 => 5,
        8 => 6,
        _ => 0,
    }
}

/// How a row shows its value; the kinds and counts must stay as Purple-lib first saw them.
pub struct RowShape {
    pub kind: u32,
    /// Labels rows: the label count. Numbers and text rows: the value count.
    pub count: u32,
    pub first: i32,
    pub step: i32,
    pub flags: u32,
}
const fn labels_shape(count: u32) -> RowShape {
    RowShape { kind: ROW_LABELS, count, first: 0, step: 0, flags: 0 }
}
pub const fn shape(row: u32) -> RowShape {
    match row {
        ROW_STYLE | ROW_PREVIEW => labels_shape(3),
        // 50%..300% in steps of 10: the ImGui slider's 0.5..3.0.
        ROW_SIZE => RowShape { kind: ROW_NUMBERS, count: 26, first: 50, step: 10, flags: ROW_NO_WRAP },
        // 0.4 s..2.0 s in steps of 0.1, shown through DURATION_TEXT.
        ROW_DURATION => RowShape { kind: ROW_TEXT_KIND, count: 17, first: 0, step: 0, flags: ROW_NO_WRAP },
        // 0%..200% in steps of 10.
        ROW_SPREAD => RowShape { kind: ROW_NUMBERS, count: 21, first: 0, step: 10, flags: ROW_NO_WRAP },
        ROW_RESTORE => RowShape { kind: ROW_ACTION, count: 1, first: 0, step: 0, flags: 0 },
        _ => labels_shape(2),
    }
}
/// Rows whose value is written to the INI; the preview rows are session-only.
pub const fn saved(row: u32) -> bool {
    row <= ROW_SPREAD
}

// `x` rounded to the nearest whole number, 0 for anything not positive or not finite. core has no
// f32::round in this toolchain's no_std build.
fn nearest(x: f32) -> u32 {
    if x.is_finite() && x > 0.0 { (x + 0.5) as u32 } else { 0 }
}
/// The value Purple-lib shows for a row: a label index or a step count, the nearest step to a value
/// the ImGui slider set between steps.
pub fn value(s: Settings, row: u32) -> u32 {
    let last = shape(row).count.saturating_sub(1);
    match row {
        ROW_ENABLED => s.enabled as u32,
        ROW_STYLE => match s.style {
            Style::Wilds => 0,
            Style::Rise => 1,
            Style::Classic => 2,
        },
        ROW_SIZE => nearest(s.scale * 10.0).saturating_sub(5).min(last),
        ROW_DURATION => nearest(s.lifetime * 10.0).saturating_sub(4).min(last),
        ROW_SPREAD => nearest(s.spread * 10.0).min(last),
        ROW_PREVIEW => s.preview as u32,
        ROW_RANDOM => s.random as u32,
        _ => 0,
    }
}
/// The settings with one row set to `value`; None for a value out of range or a row without one.
/// Division by 10 gives the same f32 as the decimal literal, so 0.9 s is the default's 0.9.
pub fn with_value(mut s: Settings, row: u32, value: u32) -> Option<Settings> {
    if row >= ROW_RESTORE || value >= shape(row).count {
        return None;
    }
    match row {
        ROW_ENABLED => s.enabled = value == 1,
        ROW_STYLE => s.style = [Style::Wilds, Style::Rise, Style::Classic][value as usize],
        ROW_SIZE => s.scale = (5 + value) as f32 / 10.0,
        ROW_DURATION => s.lifetime = (4 + value) as f32 / 10.0,
        ROW_SPREAD => s.spread = value as f32 / 10.0,
        ROW_PREVIEW => s.preview = [Preview::Single, Preview::Burst, Preview::Multi][value as usize],
        _ => s.random = value == 1,
    }
    Some(s)
}

/// Purple-lib keeps every pointer it is given, so all text is static. The game's own Off/On
/// (StartMenuMsg 175/176) is reused from Please do Better. Style names are the games' titles and
/// stay in English.
pub struct RowText {
    pub name: [&'static CStr; LANGUAGES],
    pub help: [&'static CStr; LANGUAGES],
}
pub static ROW_TEXT: [RowText; ROWS as usize] = [
    RowText {
        name: [c"Show damage numbers", c"Afficher les dégâts", c"Mostrar daño", c"Schadenszahlen zeigen", c"Mostra i danni", c"顯示傷害數字", c"显示伤害数字"],
        help: [
            c"Show the damage of each hit where it\r\nlanded.",
            c"Affiche les dégâts de chaque coup là\r\noù il a touché.",
            c"Muestra el daño de cada golpe donde\r\nimpactó.",
            c"Zeigt den Schaden jedes Treffers dort,\r\nwo er getroffen hat.",
            c"Mostra il danno di ogni colpo dove\r\nha colpito.",
            c"在命中位置顯示每次攻擊的傷害。",
            c"在命中位置显示每次攻击的伤害。",
        ],
    },
    RowText {
        name: [c"Style", c"Style", c"Estilo", c"Stil", c"Stile", c"樣式", c"样式"],
        help: [
            c"How the numbers look: Wilds, Rise or\r\nClassic.",
            c"L'aspect des chiffres : Wilds, Rise\r\nou Classic.",
            c"El aspecto de los números: Wilds, Rise\r\no Classic.",
            c"Wie die Zahlen aussehen: Wilds, Rise\r\noder Classic.",
            c"L'aspetto dei numeri: Wilds, Rise o\r\nClassic.",
            c"數字的外觀：Wilds、Rise或Classic。",
            c"数字的外观：Wilds、Rise或Classic。",
        ],
    },
    RowText {
        name: [c"Number size", c"Taille des chiffres", c"Tamaño de los números", c"Zahlengröße", c"Dimensione numeri", c"數字大小", c"数字大小"],
        help: [
            c"Make the numbers bigger or smaller.",
            c"Agrandit ou réduit les chiffres.",
            c"Agranda o reduce los números.",
            c"Macht die Zahlen größer oder kleiner.",
            c"Ingrandisce o riduce i numeri.",
            c"放大或縮小數字。",
            c"放大或缩小数字。",
        ],
    },
    RowText {
        name: [c"Duration", c"Durée", c"Duración", c"Dauer", c"Durata", c"顯示時間", c"显示时间"],
        help: [
            c"How long each number stays on screen.",
            c"Durée d'affichage de chaque chiffre.",
            c"Cuánto tiempo se ve cada número.",
            c"Wie lange jede Zahl zu sehen ist.",
            c"Quanto resta a schermo ogni numero.",
            c"每個數字在畫面上停留的時間。",
            c"每个数字在画面上停留的时间。",
        ],
    },
    RowText {
        name: [c"Spread", c"Écartement", c"Separación", c"Abstand", c"Distanza", c"分散", c"分散"],
        help: [
            c"How far apart numbers move when they\r\nwould overlap.",
            c"Jusqu'où les chiffres s'écartent quand\r\nils se chevauchent.",
            c"Cuánto se separan los números cuando\r\nse superponen.",
            c"Wie weit Zahlen ausweichen, wenn sie\r\nsich überlagern.",
            c"Quanto si allontanano i numeri quando\r\nsi sovrappongono.",
            c"數字重疊時彼此錯開的距離。",
            c"数字重叠时彼此错开的距离。",
        ],
    },
    RowText {
        name: [c"Preview mode", c"Mode d'aperçu", c"Modo de vista previa", c"Vorschaumodus", c"Modalità anteprima", c"預覽模式", c"预览模式"],
        help: [
            c"How the preview on the left shows\r\nhits. Real numbers don't change.",
            c"Comment l'aperçu montre les coups. Les\r\nvrais chiffres ne changent pas.",
            c"Cómo muestra los golpes la vista previa.\r\nLos números reales no cambian.",
            c"Wie die Vorschau links Treffer zeigt.\r\nEchte Zahlen ändern sich nicht.",
            c"Come l'anteprima a sinistra mostra i\r\ncolpi. I numeri veri non cambiano.",
            c"左側預覽顯示攻擊的方式。\r\n不影響實際的數字。",
            c"左侧预览显示攻击的方式。\r\n不影响实际的数字。",
        ],
    },
    RowText {
        name: [c"Random hit types", c"Types de coups aléatoires", c"Tipos de golpe aleatorios", c"Zufällige Trefferarten", c"Tipi di colpo casuali", c"隨機攻擊類型", c"随机攻击类型"],
        help: [
            c"Give each preview hit a random type.",
            c"Donne un type au hasard à chaque coup\r\nde l'aperçu.",
            c"Da un tipo al azar a cada golpe de la\r\nvista previa.",
            c"Gibt jedem Vorschautreffer eine\r\nzufällige Art.",
            c"Dà un tipo casuale a ogni colpo\r\ndell'anteprima.",
            c"讓每次預覽攻擊隨機決定類型。",
            c"让每次预览攻击随机决定类型。",
        ],
    },
    RowText {
        name: [c"Restore defaults", c"Valeurs par défaut", c"Valores por defecto", c"Standardwerte", c"Valori predefiniti", c"恢復預設值", c"恢复默认值"],
        help: [
            c"Set every option on this page back to\r\nits default.",
            c"Remet chaque option de cette page à sa\r\nvaleur par défaut.",
            c"Devuelve cada opción de esta página a\r\nsu valor por defecto.",
            c"Setzt jede Option dieser Seite auf\r\nihren Standardwert zurück.",
            c"Riporta ogni opzione di questa pagina\r\nal valore predefinito.",
            c"將此頁所有選項恢復為預設值。",
            c"将此页所有选项恢复为默认值。",
        ],
    },
];

/// Show damage numbers and Random hit types: the game's own Off/On (StartMenuMsg 175/176).
pub static OFF_ON: [[&CStr; 2]; LANGUAGES] = [
    [c"Off", c"On"],
    [c"Désactivé", c"Activé"],
    [c"No", c"Sí"],
    [c"Aus", c"An"],
    [c"No", c"Sì"],
    [c"不使用", c"使用"],
    [c"不使用", c"使用"],
];
/// Style labels in row-value order (Wilds, Rise, Classic, as in the ImGui combo).
pub static STYLES: [&CStr; 3] = [c"Wilds", c"Rise", c"Classic"];
pub static PREVIEWS: [[&CStr; 3]; LANGUAGES] = [
    [c"Single", c"Burst", c"Multi"],
    [c"Unique", c"Rafale", c"Multiple"],
    [c"Único", c"Ráfaga", c"Múltiple"],
    [c"Einzeln", c"Salve", c"Mehrfach"],
    [c"Singolo", c"Raffica", c"Multiplo"],
    [c"單發", c"連發", c"同時"],
    [c"单发", c"连发", c"同时"],
];
/// The action row's text on the right.
pub static RESTORE: [[&CStr; 1]; LANGUAGES] = [
    [c"Restore"],
    [c"Rétablir"],
    [c"Restablecer"],
    [c"Standard"],
    [c"Ripristina"],
    [c"恢復"],
    [c"恢复"],
];
/// A row's labels in a language; empty for numbers and text rows.
pub fn labels(row: u32, language: usize) -> &'static [&'static CStr] {
    match row {
        ROW_ENABLED | ROW_RANDOM => &OFF_ON[language],
        ROW_STYLE => &STYLES,
        ROW_PREVIEW => &PREVIEWS[language],
        ROW_RESTORE => &RESTORE[language],
        _ => &[],
    }
}
/// The numbers rows' suffix, the same in every language.
pub const PERCENT: &CStr = c"%";

/// Duration's values 0.4 s..2.0 s: English, comma decimals (French, Spanish, German, Italian) and
/// Chinese.
pub static DURATION_TEXT: [[&CStr; 17]; 3] = [
    [c"0.4 s", c"0.5 s", c"0.6 s", c"0.7 s", c"0.8 s", c"0.9 s", c"1.0 s", c"1.1 s", c"1.2 s", c"1.3 s", c"1.4 s", c"1.5 s", c"1.6 s", c"1.7 s", c"1.8 s", c"1.9 s", c"2.0 s"],
    [c"0,4 s", c"0,5 s", c"0,6 s", c"0,7 s", c"0,8 s", c"0,9 s", c"1,0 s", c"1,1 s", c"1,2 s", c"1,3 s", c"1,4 s", c"1,5 s", c"1,6 s", c"1,7 s", c"1,8 s", c"1,9 s", c"2,0 s"],
    [c"0.4秒", c"0.5秒", c"0.6秒", c"0.7秒", c"0.8秒", c"0.9秒", c"1.0秒", c"1.1秒", c"1.2秒", c"1.3秒", c"1.4秒", c"1.5秒", c"1.6秒", c"1.7秒", c"1.8秒", c"1.9秒", c"2.0秒"],
];
pub fn duration_text(language: usize, value: u32) -> Option<&'static CStr> {
    let form = match language {
        0 => 0,
        1..=4 => 1,
        _ => 2,
    };
    DURATION_TEXT[form].get(value as usize).copied()
}

/// The page's own help, which the help button opens on the Mod Options page
/// (`purplelib_add_set_help`, category 0): one topic per setting row, titled with the row's name, in
/// row order. A page has five lines at most, about 40 columns a line (a CJK character takes two).
pub const HELP_CATEGORY: u32 = 0;
pub const TOPICS: usize = 7;
/// Pages per topic, for the `Help` records; a host test checks it against `HELP_TEXT`.
pub const HELP_PAGES: [u32; TOPICS] = [2, 1, 1, 1, 1, 2, 1];
/// `HELP_TEXT[topic][language]`: the topic's pages. Topic n explains row n.
pub static HELP_TEXT: [[&[&CStr]; LANGUAGES]; TOPICS] = [
    [
        &[
            c"Shows a number for each hit at the spot\r\nit landed: your attacks, your Palico's,\r\nkinsect hits, poison, blast and mount\r\nknockdowns. Off hides them; the preview\r\nstill shows numbers.",
            c"Yellow numbers hit a weak point. Gray\r\nnumbers bounced off or are poison.\r\nCritical hits get an extra marker, which\r\ndiffers for positive and negative\r\naffinity.",
        ],
        &[
            c"Affiche un chiffre pour chaque coup là\r\noù il a touché : vos attaques, votre\r\nPalico, le Kinsecte, poison, explosion\r\net chute de monture. Désactivé les\r\ncache ; l'aperçu les montre toujours.",
            c"Jaune : point faible touché. Gris :\r\ncoup dévié ou poison. Les coups\r\ncritiques ont une marque en plus, qui\r\ndiffère pour l'affinité positive et\r\nnégative.",
        ],
        &[
            c"Muestra un número por cada golpe donde\r\nimpactó: tus ataques, tu Camarada, el\r\nKinsecto, veneno, nitro y la caída de\r\nmonta. Desactivado los oculta; la vista\r\nprevia los sigue mostrando.",
            c"Amarillo: golpe en punto débil. Gris:\r\ngolpe rebotado o veneno. Los críticos\r\nllevan una marca extra, distinta para\r\nafinidad positiva y negativa.",
        ],
        &[
            c"Zeigt für jeden Treffer eine Zahl, wo\r\ner traf: deine Angriffe, dein Kamerad,\r\nKinsekt, Gift, Explosion und Sturz beim\r\nReiten. Aus blendet sie aus; die\r\nVorschau zeigt sie weiter.",
            c"Gelb: Treffer am Schwachpunkt. Grau:\r\nabgeprallt oder Gift. Kritische Treffer\r\nhaben eine zusätzliche Markierung, je\r\nnach positiver oder negativer Affinität.",
        ],
        &[
            c"Mostra un numero per ogni colpo dove\r\nha colpito: i tuoi attacchi, il tuo\r\nCompagno, il Kinsetto, veleno, esplosivo\r\ne caduta in monta. Disattivato li\r\nnasconde; l'anteprima li mostra sempre.",
            c"Giallo: colpo su punto debole. Grigio:\r\ncolpo rimbalzato o veleno. I critici\r\nhanno un segno in più, diverso per\r\naffinità positiva e negativa.",
        ],
        &[
            c"在命中位置為每次攻擊顯示數字：\r\n自己的攻擊、隨從、獵蟲、\r\n毒、爆破與騎乘倒地。\r\n關閉時不顯示，但預覽仍會顯示。",
            c"黃色表示命中弱點，\r\n灰色表示被彈開或毒傷害。\r\n會心攻擊另有標記，\r\n正負會心的標記不同。",
        ],
        &[
            c"在命中位置为每次攻击显示数字：\r\n自己的攻击、随从、猎虫、\r\n毒、爆破与骑乘倒地。\r\n关闭时不显示，但预览仍会显示。",
            c"黄色表示命中弱点，\r\n灰色表示被弹开或毒伤害。\r\n会心攻击另有标记，\r\n正负会心的标记不同。",
        ],
    ],
    [
        &[c"Wilds: outlined digits, and a glowing\r\nstar on critical hits.\r\nRise: a slanted slash on critical hits,\r\nand larger weak-point numbers.\r\nClassic: straight critical markers."],
        &[c"Wilds : chiffres détourés et étoile\r\nlumineuse sur les critiques.\r\nRise : entaille oblique sur les\r\ncritiques, points faibles plus grands.\r\nClassic : marques droites."],
        &[c"Wilds: números con contorno y una\r\nestrella brillante en los críticos.\r\nRise: un tajo oblicuo en los críticos y\r\nnúmeros más grandes en puntos débiles.\r\nClassic: marcas rectas en críticos."],
        &[c"Wilds: umrandete Ziffern und ein\r\nleuchtender Stern bei Kritischen.\r\nRise: schräger Schnitt bei Kritischen,\r\ngrößere Zahlen am Schwachpunkt.\r\nClassic: gerade Markierungen."],
        &[c"Wilds: cifre bordate e una stella\r\nluminosa sui critici.\r\nRise: un taglio obliquo sui critici e\r\nnumeri più grandi sui punti deboli.\r\nClassic: segni dritti sui critici."],
        &[c"Wilds：有外框的數字，\r\n會心時顯示發光的星形。\r\nRise：會心時顯示斜向斬痕，\r\n命中弱點的數字較大。\r\nClassic：會心時顯示直線標記。"],
        &[c"Wilds：有外框的数字，\r\n会心时显示发光的星形。\r\nRise：会心时显示斜向斩痕，\r\n命中弱点的数字较大。\r\nClassic：会心时显示直线标记。"],
    ],
    [
        &[c"Sets the size of every number, from 50%\r\nto 300%. Sizes also follow the height of\r\nthe screen."],
        &[c"Règle la taille de tous les chiffres,\r\nde 50 % à 300 %. Elle suit aussi la\r\nhauteur de l'écran."],
        &[c"Ajusta el tamaño de todos los números,\r\nde 50 % a 300 %. También sigue la\r\naltura de la pantalla."],
        &[c"Stellt die Größe aller Zahlen ein, von\r\n50 % bis 300 %. Sie folgt auch der\r\nBildschirmhöhe."],
        &[c"Imposta la dimensione di tutti i\r\nnumeri, dal 50% al 300%. Segue anche\r\nl'altezza dello schermo."],
        &[c"設定所有數字的大小，\r\n範圍為50%至300%。\r\n大小也會隨畫面高度調整。"],
        &[c"设定所有数字的大小，\r\n范围为50%至300%。\r\n大小也会随画面高度调整。"],
    ],
    [
        &[c"How long each number stays on screen,\r\nfrom 0.4 to 2.0 seconds. Each number\r\nfades out at the end."],
        &[c"Durée d'affichage de chaque chiffre,\r\nde 0,4 à 2,0 secondes. Chaque chiffre\r\ndisparaît en fondu à la fin."],
        &[c"Cuánto tiempo se ve cada número, de\r\n0,4 a 2,0 segundos. Cada número se\r\ndesvanece al final."],
        &[c"Wie lange jede Zahl zu sehen ist, von\r\n0,4 bis 2,0 Sekunden. Am Ende blendet\r\nsie aus."],
        &[c"Quanto resta a schermo ogni numero, da\r\n0,4 a 2,0 secondi. Alla fine ogni\r\nnumero sfuma."],
        &[c"每個數字在畫面上停留的時間，\r\n範圍為0.4至2.0秒。\r\n數字會在最後淡出。"],
        &[c"每个数字在画面上停留的时间，\r\n范围为0.4至2.0秒。\r\n数字会在最后淡出。"],
    ],
    [
        &[c"When numbers would overlap, later ones\r\nmove to a free spot nearby. Spread sets\r\nhow far they may move: 0% stacks them on\r\nthe hit, 200% spreads them widely. The\r\ndefault is 80%."],
        &[c"Si des chiffres se chevauchent, les\r\nsuivants vont à une place libre proche.\r\nL'écartement règle la distance : 0 %\r\nles empile, 200 % les disperse.\r\nPar défaut : 80 %."],
        &[c"Si los números se superponen, los\r\nsiguientes van a un hueco cercano. La\r\nseparación fija cuánto: 0 % los apila,\r\n200 % los dispersa mucho.\r\nPor defecto: 80 %."],
        &[c"Überlagern sich Zahlen, weichen spätere\r\nan eine freie Stelle in der Nähe aus.\r\nAbstand legt fest, wie weit: 0 % stapelt\r\nsie, 200 % verteilt sie weit.\r\nStandard: 80 %."],
        &[c"Se i numeri si sovrappongono, i\r\nsuccessivi vanno in un posto libero\r\nvicino. La distanza dice quanto: 0% li\r\nimpila, 200% li sparge molto.\r\nPredefinito: 80%."],
        &[c"數字重疊時，後出現的數字\r\n會移到附近的空位。\r\n分散決定移動的距離：\r\n0%疊在命中處，200%分得最開。\r\n預設值為80%。"],
        &[c"数字重叠时，后出现的数字\r\n会移到附近的空位。\r\n分散决定移动的距离：\r\n0%叠在命中处，200%分得最开。\r\n默认值为80%。"],
    ],
    [
        &[
            c"While this page is open, the frame on\r\nthe left loops example hits at three\r\nspots. The spots stay where they are in\r\nthe world, so turning the camera moves\r\nthem like real hits.",
            c"Single: one hit at each spot.\r\nBurst: five hits, 0.15 s apart.\r\nMulti: five hits at once, so you can see\r\nhow Spread moves them apart.\r\nThis option is not saved.",
        ],
        &[
            c"Quand cette page est ouverte, le cadre\r\nà gauche répète des coups d'exemple à\r\ntrois endroits. Ils restent fixes dans\r\nle monde : tourner la caméra les\r\ndéplace comme de vrais coups.",
            c"Unique : un coup par endroit.\r\nRafale : cinq coups, à 0,15 s d'écart.\r\nMultiple : cinq coups à la fois, pour\r\nvoir l'effet de l'écartement.\r\nCette option n'est pas enregistrée.",
        ],
        &[
            c"Con esta página abierta, el marco de la\r\nizquierda repite golpes de ejemplo en\r\ntres puntos. Están fijos en el mundo:\r\nal girar la cámara se mueven como\r\ngolpes reales.",
            c"Único: un golpe en cada punto.\r\nRáfaga: cinco golpes, cada 0,15 s.\r\nMúltiple: cinco a la vez, para ver el\r\nefecto de Separación.\r\nEsta opción no se guarda.",
        ],
        &[
            c"Solange diese Seite offen ist, zeigt\r\nder Rahmen links an drei Stellen\r\nBeispieltreffer in Schleife. Sie sind\r\nin der Welt fest: Dreht sich die Kamera,\r\nbewegen sie sich wie echte Treffer.",
            c"Einzeln: ein Treffer pro Stelle.\r\nSalve: fünf Treffer, 0,15 s Abstand.\r\nMehrfach: fünf zugleich, so siehst du,\r\nwie Abstand sie verteilt.\r\nDiese Option wird nicht gespeichert.",
        ],
        &[
            c"Con questa pagina aperta, il riquadro a\r\nsinistra ripete colpi di esempio in tre\r\npunti. Restano fermi nel mondo: girando\r\nla telecamera si muovono come colpi\r\nveri.",
            c"Singolo: un colpo per punto.\r\nRaffica: cinque colpi a 0,15 s.\r\nMultiplo: cinque insieme, per vedere\r\ncome li separa la distanza.\r\nQuesta opzione non viene salvata.",
        ],
        &[
            c"此頁開啟時，左側框內會在\r\n三個位置循環顯示示範攻擊。\r\n這些位置固定在遊戲世界中，\r\n轉動鏡頭時會像實際攻擊一樣移動。",
            c"單發：每個位置一次攻擊。\r\n連發：五次攻擊，間隔0.15秒。\r\n同時：同時五次攻擊，\r\n可看出分散的效果。\r\n此選項不會儲存。",
        ],
        &[
            c"此页开启时，左侧框内会在\r\n三个位置循环显示示范攻击。\r\n这些位置固定在游戏世界中，\r\n转动镜头时会像实际攻击一样移动。",
            c"单发：每个位置一次攻击。\r\n连发：五次攻击，间隔0.15秒。\r\n同时：同时五次攻击，\r\n可看出分散的效果。\r\n此选项不会保存。",
        ],
    ],
    [
        &[c"Off: the left spot shows weak-point hits\r\nwith positive affinity, the middle\r\nnormal hits with negative affinity, the\r\nright bounced hits. On: any type at any\r\nspot. This option is not saved."],
        &[c"Désactivé : à gauche, point faible et\r\naffinité positive ; au milieu, coup\r\nnormal et affinité négative ; à droite,\r\ncoup dévié. Activé : tout type partout.\r\nCette option n'est pas enregistrée."],
        &[c"No: izquierda, punto débil con afinidad\r\npositiva; centro, golpe normal con\r\nafinidad negativa; derecha, rebote.\r\nSí: cualquier tipo en cualquier punto.\r\nEsta opción no se guarda."],
        &[c"Aus: links Schwachpunkt mit positiver\r\nAffinität, Mitte normaler Treffer mit\r\nnegativer Affinität, rechts abgeprallt.\r\nAn: jede Art an jeder Stelle.\r\nDiese Option wird nicht gespeichert."],
        &[c"No: a sinistra punto debole con\r\naffinità positiva, al centro colpo\r\nnormale con affinità negativa, a destra\r\nrimbalzo. Sì: ogni tipo ovunque.\r\nQuesta opzione non viene salvata."],
        &[c"不使用：左為命中弱點且正會心，\r\n中為一般攻擊且負會心，\r\n右為被彈開的攻擊。\r\n使用：任何位置都可能是任何類型。\r\n此選項不會儲存。"],
        &[c"不使用：左为命中弱点且正会心，\r\n中为一般攻击且负会心，\r\n右为被弹开的攻击。\r\n使用：任何位置都可能是任何类型。\r\n此选项不会保存。"],
    ],
];

/// Display width as the help screens count it, roughly: a CJK character is two columns.
pub fn columns(text: &str) -> usize {
    text.chars().map(|c| if (c as u32) >= 0x2e80 { 2 } else { 1 }).sum()
}
