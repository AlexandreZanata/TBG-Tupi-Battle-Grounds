//! Interface text in Brazilian Portuguese (default) and English.

use bevy::prelude::*;
use tbg_moba_core::{Destination, Lane, Role, TroopKind};

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    Pt,
    En,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Title,
    Subtitle,
    PlayBots,
    PlayOnline,
    Quit,
    ChooseRole,
    BlueTeam,
    RedTeam,
    You,
    Destination,
    BuyAndSend,
    Auto,
    Hint,
    Victory,
    Defeat,
    Draw,
    PlayAgain,
    BackToMenu,
    Connecting,
    ConnectionLost,
    RoomFull,
    TowerFallen,
    Respawn,
    NotEnoughGold,
    TroopCap,
    NotEnoughEssence,
    MaxTier,
    Level,
    Tier,
    Troops,
}

impl Lang {
    pub fn t(self, key: Key) -> &'static str {
        let (pt, en) = match key {
            Key::Title => ("TBG · Arena 5v5", "TBG · 5v5 Arena"),
            Key::Subtitle => ("Vale Tupi — três rotas, uma selva, dez comandantes", "Tupi Valley — three lanes, one jungle, ten commanders"),
            Key::PlayBots => ("Treinar contra 9 bots", "Practice against 9 bots"),
            Key::PlayOnline => ("Jogar online (sala local)", "Play online (local room)"),
            Key::Quit => ("Sair", "Quit"),
            Key::ChooseRole => ("Escolha sua função", "Choose your role"),
            Key::BlueTeam => ("Equipe Azul", "Blue Team"),
            Key::RedTeam => ("Equipe Vermelha", "Red Team"),
            Key::You => ("Você", "You"),
            Key::Destination => ("Destino da Tropa", "Troop Destination"),
            Key::BuyAndSend => ("Comprar e Enviar", "Buy and Send"),
            Key::Auto => ("Automático", "Automatic"),
            Key::Hint => (
                "Botão direito: mover comandante · A: automático · Espaço: centralizar · 1–5 tropa · Z X C V destino · Enter comprar",
                "Right click: move commander · A: automatic · Space: centre · 1–5 troop · Z X C V destination · Enter buy",
            ),
            Key::Victory => ("Vitória!", "Victory!"),
            Key::Defeat => ("Derrota", "Defeat"),
            Key::Draw => ("Empate", "Draw"),
            Key::PlayAgain => ("Jogar novamente", "Play again"),
            Key::BackToMenu => ("Menu", "Menu"),
            Key::Connecting => ("Conectando à sala…", "Connecting to the room…"),
            Key::ConnectionLost => ("Conexão perdida", "Connection lost"),
            Key::RoomFull => ("Sala cheia", "Room full"),
            Key::TowerFallen => ("Torre destruída", "Tower destroyed"),
            Key::Respawn => ("renasce em", "respawns in"),
            Key::NotEnoughGold => ("Ouro insuficiente", "Not enough gold"),
            Key::TroopCap => ("Limite de tropas vivas", "Live troop limit reached"),
            Key::NotEnoughEssence => ("Essência insuficiente", "Not enough essence"),
            Key::MaxTier => ("Tropas no nível máximo", "Troops at max tier"),
            Key::Level => ("Nível", "Level"),
            Key::Tier => ("Aprimoramento", "Tier"),
            Key::Troops => ("Tropas", "Troops"),
        };
        match self {
            Lang::Pt => pt,
            Lang::En => en,
        }
    }

    pub fn role(self, role: Role) -> &'static str {
        // Role names stay in English in both languages, as players say them.
        match role {
            Role::Top => "Top",
            Role::Jungle => "Jungle",
            Role::Mid => "Mid",
            Role::Bot => "Bot",
            Role::Support => "Support",
        }
    }

    pub fn lane(self, to: Destination) -> &'static str {
        match to {
            Destination::Lane(Lane::Top) => self.role(Role::Top),
            Destination::Lane(Lane::Mid) => self.role(Role::Mid),
            Destination::Lane(Lane::Bot) => self.role(Role::Bot),
            Destination::Jungle => self.role(Role::Jungle),
        }
    }

    /// Card name and archetype subtitle.
    pub fn troop(self, troop: TroopKind) -> (&'static str, &'static str) {
        let pt = matches!(self, Lang::Pt);
        match troop {
            TroopKind::Guardian => (if pt { "Guardião" } else { "Guardian" }, "Tank"),
            TroopKind::Healer => (
                if pt { "Curandeiro" } else { "Healer" },
                if pt { "Suporte" } else { "Support" },
            ),
            TroopKind::Bard => (
                if pt { "Bardo" } else { "Bard" },
                if pt { "Suporte" } else { "Support" },
            ),
            TroopKind::Colossus => (if pt { "Colosso" } else { "Colossus" }, "Tank"),
            TroopKind::Summoner => (
                if pt { "Invocador" } else { "Summoner" },
                if pt { "Controle" } else { "Control" },
            ),
        }
    }
}

/// The UI faces: a Latin-1 Noto Sans subset as Bevy's default font slot,
/// plus the display face for titles (both OFL, see `assets/fonts/OFL.txt`).
static UI_FONT: &[u8] = include_bytes!("../assets/fonts/BattlegroundSansUi.ttf");
static DISPLAY_FONT: &[u8] = include_bytes!("../assets/fonts/BattlegroundDisplayUi.ttf");

#[derive(Resource, Clone)]
pub struct Fonts {
    pub display: Handle<Font>,
}

pub fn plugin(app: &mut App) {
    let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
    // Same slot `bevy_text` uses for its default font, so plain TextFont works.
    let _ = fonts.insert(AssetId::default(), Font::from_bytes(UI_FONT.to_vec()));
    let display = fonts.add(Font::from_bytes(DISPLAY_FONT.to_vec()));
    app.insert_resource(Fonts { display });
}
