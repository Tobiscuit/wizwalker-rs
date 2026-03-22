//! Window path constants — faithfully ported from Deimos `src/paths.py`.
//!
//! Each path is a list of window names that are followed from the root window
//! to find a specific UI element. An empty string "" means "match any child name".

/// Spiral Door Paths
pub const SPIRAL_DOOR_RESET: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "optionWindow", "leftButton"];
pub const SPIRAL_DOOR_CYCLE: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "optionWindow", "rightButton"];
pub const SPIRAL_DOOR_TELEPORT: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "teleportButton"];
pub const SPIRAL_DOOR_WORLD: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite"];
pub const SPIRAL_DOOR_SELECTED: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "selectedWorldCheckMark"];
pub const SPIRAL_DOOR: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "optionWindow"];
pub const SPIRAL_DOOR_TITLE: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "streamTitle"];
pub const SPIRAL_DOOR_EXIT: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "cancelButton"];

/// Potion Buying/Usage Paths
pub const POTION_SHOP_BASE: &[&str] = &["WorldView", "main"];
pub const POTION_BUY: &[&str] = &["WorldView", "main", "buyAction"];
pub const POTION_FILL_ALL: &[&str] = &["WorldView", "main", "fillallpotions"];
pub const POTION_EXIT: &[&str] = &["WorldView", "main", "exit"];
pub const POTION_USAGE: &[&str] = &["WorldView", "windowHUD", "btnPotions"];

/// Spellbook Paths
pub const CHECK_SPELLBOOK_OPEN: &[&str] = &["WorldView", "DeckConfiguration"];

/// Quitting/Playing Paths
pub const QUIT_BUTTON: &[&str] = &["WorldView", "DeckConfiguration", "SettingPage", "QuitButton"];
pub const CLOSE_SPELLBOOK: &[&str] = &["WorldView", "DeckConfiguration", "Close_Button"];
pub const PLAY_BUTTON: &[&str] = &["WorldView", "mainWindow", "btnPlay"];

/// Dungeon Entry Paths
pub const DUNGEON_WARNING: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];

/// Dialogue Paths
pub const ADVANCE_DIALOG: &[&str] = &["WorldView", "wndDialogMain", "btnRight"];
pub const DECLINE_QUEST: &[&str] = &["WorldView", "wndDialogMain", "btnLeft"];
pub const DIALOG_TEXT: &[&str] = &["WorldView", "wndDialogMain", "txtArea", "txtMessage"];

/// Quest Objective Path
pub const QUEST_NAME: &[&str] = &["WorldView", "windowHUD", "QuestHelperHud", "ElementWindow", "", "txtGoalName"];

/// NPC Range Popup Paths
pub const POPUP_TITLE: &[&str] = &["WorldView", "NPCRangeWin", "wndTitleBackground", "NPCRangeTxtTitle"];
pub const POPUP_MSGTEXT: &[&str] = &["WorldView", "NPCRangeWin", "imgBackground", "NPCRangeTxtMessage"];

/// Team Up Paths
pub const TEAM_UP_BUTTON: &[&str] = &["WorldView", "NPCRangeWin", "imgBackground", "TeamUpButton"];
pub const TEAM_UP_CONFIRM: &[&str] = &["WorldView", "TeamUpConfirmationWindow", "TeamUpConfirmationBackground", "TeamUpButton"];

/// NPC Range Path
pub const NPC_RANGE: &[&str] = &["WorldView", "NPCRangeWin"];

/// Cancel Chest Roll Path
pub const CANCEL_CHEST_ROLL: &[&str] = &["WorldView", "Container", "background", "", "CancelButton"];

/// Missing Area Menu Paths
pub const MISSING_AREA: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow"];
pub const MISSING_AREA_RETRY: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "RetryBtn"];

/// Willcast Card Paths
pub const WILLCAST: &[&str] = &["WorldView", "PlanningPhase", "Alignment", "PlanningPhaseSubWindow", "SpellSelection", "Hand", "PetCard", "PetCardWindow"];

/// Exit Shop Paths
pub const EXIT_RECIPE_SHOP: &[&str] = &["WorldView", "", "Exit"];
pub const EXIT_EQUIPMENT_SHOP: &[&str] = &["WorldView", "shopGUI", "buyWindow", "exit"];
pub const EXIT_SNACK_SHOP: &[&str] = &["WorldView", "ShoppingPetSnackWindow", "buyWindow", "exit"];
pub const EXIT_REAGENT_SHOP: &[&str] = &["WorldView", "ShoppingReagentWindow", "buyWindow", "exit"];
pub const EXIT_TC_VENDOR: &[&str] = &["WorldView", "main", "exit"];
pub const EXIT_MINIGAME_SIGIL: &[&str] = &["WorldView", "mainwindow", "exit"];

/// Cancel Quest Menus
pub const CANCEL_MULTIPLE_QUEST_MENU: &[&str] = &["WorldView", "NPCServicesWin", "wndDialogMain", "Exit"];
pub const CANCEL_SPELL_VENDOR: &[&str] = &["WorldView", "NPCTrainingGUI", "TrainingSelection", "Exit"];

/// Resume Instance Button
pub const RESUME_INSTANCE: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "ResumeInstanceButton"];

/// Exit Dungeon Path
pub const EXIT_DUNGEON: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];

/// Friend Busy / Dungeon Reset Path
pub const FRIEND_IS_BUSY_AND_DUNGEON_RESET: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "rightButton"];

/// Friend List / Friend Popup Paths
pub const ADD_REMOVE_FRIEND: &[&str] = &["WorldView", "windowHUD", "wndCharacter", "ButtonLayout", "btnAddRemoveFriend"];
pub const CONFIRM_SEND_FRIEND_REQUEST: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];

/// Teleport Mark Recall Path
pub const TELEPORT_MARK_RECALL: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "RecallButton"];
pub const TELEPORT_MARK_RECALL_TIMER: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "RecallButton", "txtRecallTimer"];
pub const DUNGEON_RECALL: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "ResumeInstanceButton"];

/// Chat Window
pub const CHAT_WINDOW: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "chatLogContainer", "chatLogInnerContainer", "chatLog"];

/// Backpack/Equipment Paths
pub const BACKPACK_IS_VISIBLE: &[&str] = &["WorldView", "DeckConfiguration", "InventorySpellbookPage", "EquipmentManager"];
pub const EQUIPMENT_SET_MANAGER_TITLE: &[&str] = &["WorldView", "DeckConfiguration", "EquipmentManager", "top_scroll", "title"];
pub const BACKPACK_TITLE: &[&str] = &["WorldView", "DeckConfiguration", "InventorySpellbookPage", "top_scroll", "title"];

/// Pet Game Paths
pub const PET_FEED_WINDOW_VISIBLE: &[&str] = &["WorldView", "PetGameTracks"];
pub const PET_FEED_WINDOW_CANCEL_BUTTON: &[&str] = &["WorldView", "PetGameTracks", "btnBack"];
pub const PLAY_DANCE_GAME_BUTTON: &[&str] = &["WorldView", "PetGameTracks", "btnNext"];
pub const SKIP_PET_GAME_BUTTON: &[&str] = &["WorldView", "PetGameTracks", "SkipGameButton"];
pub const WON_PET_GAME_REWARDS_WINDOW: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards"];
pub const SKIPPED_PET_GAME_REWARDS_WINDOW: &[&str] = &["WorldView", "", "PetGameRewards"];
pub const WON_PET_GAME_CONTINUE_BUTTON: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "btnNext"];
pub const SKIPPED_PET_GAME_CONTINUE_BUTTON: &[&str] = &["WorldView", "", "PetGameRewards", "btnNext"];
pub const WON_PET_GAME_PLAY_AGAIN_BUTTON: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "btnPlayAgain"];
pub const WON_FINISH_PET_BUTTON: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "btnBack"];
pub const SKIPPED_FINISH_PET_BUTTON: &[&str] = &["WorldView", "", "PetGameRewards", "btnBack"];
pub const PET_FEED_ENERGY_COST: &[&str] = &["WorldView", "PetGameTracks", "wndBkgEnergy", "txtEnergyCost"];
pub const PET_FEED_YOUR_ENERGY: &[&str] = &["WorldView", "PetGameTracks", "wndBkgEnergy", "txtYourEnergy"];
pub const DANCE_GAME_ACTION_TEXT: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameDance", "wndControls", "wndActionBkg", "txtAction"];

/// Energy Amount
pub const ENERGY_AMOUNT: &[&str] = &["WorldView", "DeckConfiguration", "", "ControlSprite", "wndEnergyFrame", "Layout", "txtEnergy"];

/// Exit Wysteria Tournament
pub const EXIT_WYSTERIA_TOURNAMENT: &[&str] = &["WorldView", "TournamentRanking", "exit"];

/// Zafaria Class Picture Exit
pub const EXIT_ZAFARIA_CLASS_PICTURE: &[&str] = &["WorldView", "ClassPicture", "exit"];

/// Exit Avalon Badge Popup
pub const AVALON_BADGE_EXIT: &[&str] = &["WorldView", "HelpHousingTips2", "toolbar", "exit"];

/// Pet Level Up
pub const EXIT_PET_LEVELED_UP: &[&str] = &["WorldView", "PetLevelUpWindow", "wndPetLevelBkg", "btnPetLevelClose"];

// ── Pet Game Specific Paths (ported from Deimos paths.py) ─────────

/// Snack selection in skipped-game mode
pub const SKIPPED_FIRST_PET_SNACK: &[&str] = &["WorldView", "", "PetGameRewards", "wndPetFeed", "wndSnack", "Slot0"];
pub const WON_FIRST_PET_SNACK: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "wndPetFeed", "wndSnack", "Slot0"];

/// Continue and feed buttons
pub const SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON: &[&str] = &["WorldView", "", "PetGameRewards", "btnNext"];
pub const WON_PET_GAME_CONTINUE_AND_FEED_BUTTON: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "btnNext"];

/// Leveled up windows
pub const SKIPPED_PET_LEVELED_UP_WINDOW: &[&str] = &["WorldView", "", "PetGameRewards", "PetLevelUpWindow"];
pub const WON_PET_LEVELED_UP_WINDOW: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "PetLevelUpWindow"];
pub const EXIT_SKIPPED_PET_LEVELED_UP: &[&str] = &["WorldView", "", "PetGameRewards", "PetLevelUpWindow", "wndPetLevelBkg", "btnPetLevelClose"];
pub const EXIT_WON_PET_LEVELED_UP: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "PetLevelUpWindow", "wndPetLevelBkg", "btnPetLevelClose"];

/// Wizard City dance game selection
pub const WIZARD_CITY_DANCE_GAME: &[&str] = &["WorldView", "PetGameTracks", "wndGameList", "wndBkgGames", "wndGame0"];

/// Dance game action textbox
pub const DANCE_GAME_ACTION_TEXTBOX: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameDance", "wndControls", "wndActionBkg", "txtAction"];

/// Cancel / Accept friend request
pub const CONFIRM_ACCEPT_FRIEND_REQUEST: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];
pub const CLOSE_REAL_FRIEND_LIST_BUTTON: &[&str] = &["WorldView", "QuickChatTabWindow", "TabBackground", "btnClose"];

/// Quest name path
pub const QUEST_NAME_PATH: &[&str] = &["WorldView", "windowHUD", "QuestHelperHud", "ElementWindow", "", "txtGoalName"];

/// Potion exit path
pub const POTION_EXIT_PATH: &[&str] = &["WorldView", "main", "exit"];

/// NPC Range path
pub const NPC_RANGE_PATH: &[&str] = &["WorldView", "NPCRangeWin"];

// ── _PATH suffix aliases (for consistency with auto_pet/questing naming) ──

pub const PET_FEED_WINDOW_VISIBLE_PATH: &[&str] = PET_FEED_WINDOW_VISIBLE;
pub const PET_FEED_WINDOW_CANCEL_BUTTON_PATH: &[&str] = PET_FEED_WINDOW_CANCEL_BUTTON;
pub const SKIP_PET_GAME_BUTTON_PATH: &[&str] = SKIP_PET_GAME_BUTTON;
pub const PLAY_DANCE_GAME_BUTTON_PATH: &[&str] = PLAY_DANCE_GAME_BUTTON;
pub const SKIPPED_PET_GAME_REWARDS_WINDOW_PATH: &[&str] = SKIPPED_PET_GAME_REWARDS_WINDOW;
pub const SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH: &[&str] = SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON;
pub const SKIPPED_FIRST_PET_SNACK_PATH: &[&str] = SKIPPED_FIRST_PET_SNACK;
pub const SKIPPED_PET_LEVELED_UP_WINDOW_PATH: &[&str] = SKIPPED_PET_LEVELED_UP_WINDOW;
pub const EXIT_SKIPPED_PET_LEVELED_UP_PATH: &[&str] = EXIT_SKIPPED_PET_LEVELED_UP;
pub const WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH: &[&str] = WON_PET_GAME_CONTINUE_AND_FEED_BUTTON;
pub const WON_PET_LEVELED_UP_WINDOW_PATH: &[&str] = WON_PET_LEVELED_UP_WINDOW;
pub const EXIT_WON_PET_LEVELED_UP_PATH: &[&str] = EXIT_WON_PET_LEVELED_UP;
pub const WON_FIRST_PET_SNACK_PATH: &[&str] = WON_FIRST_PET_SNACK;
pub const WON_PET_GAME_REWARDS_WINDOW_PATH: &[&str] = WON_PET_GAME_REWARDS_WINDOW;
pub const WIZARD_CITY_DANCE_GAME_PATH: &[&str] = WIZARD_CITY_DANCE_GAME;
pub const DANCE_GAME_ACTION_TEXTBOX_PATH: &[&str] = DANCE_GAME_ACTION_TEXTBOX;
pub const PET_FEED_WINDOW_ENERGY_COST_TEXTBOX_PATH: &[&str] = PET_FEED_ENERGY_COST;
pub const PET_FEED_WINDOW_YOUR_ENERGY_TEXTBOX_PATH: &[&str] = PET_FEED_YOUR_ENERGY;

/// Spiral door teleport path alias
pub const SPIRAL_DOOR_TELEPORT_PATH: &[&str] = SPIRAL_DOOR_TELEPORT;

/// Exit shop path aliases
pub const EXIT_RECIPE_SHOP_PATH: &[&str] = EXIT_RECIPE_SHOP;
pub const EXIT_EQUIPMENT_SHOP_PATH: &[&str] = EXIT_EQUIPMENT_SHOP;
pub const EXIT_SNACK_SHOP_PATH: &[&str] = EXIT_SNACK_SHOP;
pub const EXIT_REAGENT_SHOP_PATH: &[&str] = EXIT_REAGENT_SHOP;
pub const EXIT_TC_VENDOR_PATH: &[&str] = EXIT_TC_VENDOR;
pub const EXIT_MINIGAME_SIGIL_PATH: &[&str] = EXIT_MINIGAME_SIGIL;
pub const EXIT_WYSTERIA_TOURNAMENT_PATH: &[&str] = EXIT_WYSTERIA_TOURNAMENT;
pub const EXIT_ZAFARIA_CLASS_PICTURE_BUTTON_PATH: &[&str] = EXIT_ZAFARIA_CLASS_PICTURE;
pub const EXIT_PET_LEVELED_UP_BUTTON_PATH: &[&str] = EXIT_PET_LEVELED_UP;
pub const AVALON_BADGE_EXIT_BUTTON_PATH: &[&str] = AVALON_BADGE_EXIT;
pub const CANCEL_MULTIPLE_QUEST_MENU_PATH: &[&str] = CANCEL_MULTIPLE_QUEST_MENU;
pub const CANCEL_SPELL_VENDOR_PATH: &[&str] = CANCEL_SPELL_VENDOR;
pub const EXIT_DUNGEON_PATH: &[&str] = EXIT_DUNGEON;
pub const FRIEND_IS_BUSY_AND_DUNGEON_RESET_PATH: &[&str] = FRIEND_IS_BUSY_AND_DUNGEON_RESET;
pub const DUNGEON_RECALL_PATH: &[&str] = DUNGEON_RECALL;
pub const CLOSE_SPELLBOOK_PATH: &[&str] = CLOSE_SPELLBOOK;
pub const CHECK_SPELLBOOK_OPEN_PATH: &[&str] = CHECK_SPELLBOOK_OPEN;
pub const DUNGEON_WARNING_PATH: &[&str] = DUNGEON_WARNING;
pub const POPUP_MSGTEXT_PATH: &[&str] = POPUP_MSGTEXT;
pub const POTION_USAGE_PATH: &[&str] = POTION_USAGE;
