//! Window path constants — faithfully ported from Deimos `src/paths.py`.
//!
//! Each path is a list of window names that are followed from the root window
//! to find a specific UI element. An empty string "" means "match any child name".

// ── Spiral Door Paths ───────────────────────────────────────────────

pub const SPIRAL_DOOR_RESET_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "optionWindow", "leftButton"];
pub const SPIRAL_DOOR_CYCLE_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "optionWindow", "rightButton"];
pub const SPIRAL_DOOR_TELEPORT_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "teleportButton"];
pub const SPIRAL_DOOR_WORLD_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite"];
pub const SPIRAL_DOOR_SELECTED_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "selectedWorldCheckMark"];
pub const SPIRAL_DOOR_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "optionWindow"];
pub const SPIRAL_DOOR_TITLE_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "streamTitle"];
pub const SPIRAL_DOOR_EXIT_PATH: &[&str] = &["WorldView", "", "messageBoxBG", "ControlSprite", "cancelButton"];

// ── Potion Buying/Usage Paths ───────────────────────────────────────

pub const POTION_SHOP_BASE_PATH: &[&str] = &["WorldView", "main"];
pub const POTION_BUY_PATH: &[&str] = &["WorldView", "main", "buyAction"];
pub const POTION_FILL_ALL_PATH: &[&str] = &["WorldView", "main", "fillallpotions"];
pub const POTION_EXIT_PATH: &[&str] = &["WorldView", "main", "exit"];
pub const POTION_USAGE_PATH: &[&str] = &["WorldView", "windowHUD", "btnPotions"];

// ── Spellbook Paths ─────────────────────────────────────────────────

pub const CHECK_SPELLBOOK_OPEN_PATH: &[&str] = &["WorldView", "DeckConfiguration"];

// ── Quitting/Playing Paths ──────────────────────────────────────────

pub const QUIT_BUTTON_PATH: &[&str] = &["WorldView", "DeckConfiguration", "SettingPage", "QuitButton"];
pub const CLOSE_SPELLBOOK_PATH: &[&str] = &["WorldView", "DeckConfiguration", "Close_Button"];
pub const PLAY_BUTTON_PATH: &[&str] = &["WorldView", "mainWindow", "btnPlay"];

// ── Dungeon Entry Paths ─────────────────────────────────────────────

pub const DUNGEON_WARNING_PATH: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];

// ── Dialogue Paths ──────────────────────────────────────────────────

pub const ADVANCE_DIALOG_PATH: &[&str] = &["WorldView", "wndDialogMain", "btnRight"];
pub const DECLINE_QUEST_PATH: &[&str] = &["WorldView", "wndDialogMain", "btnLeft"];
pub const DIALOG_TEXT_PATH: &[&str] = &["WorldView", "wndDialogMain", "txtArea", "txtMessage"];

// ── Quest Objective Path ────────────────────────────────────────────

pub const QUEST_NAME_PATH: &[&str] = &["WorldView", "windowHUD", "QuestHelperHud", "ElementWindow", "", "txtGoalName"];

// ── NPC Range Popup Paths ───────────────────────────────────────────

pub const POPUP_TITLE_PATH: &[&str] = &["WorldView", "NPCRangeWin", "wndTitleBackground", "NPCRangeTxtTitle"];

// ── Team Up Paths ───────────────────────────────────────────────────

pub const TEAM_UP_BUTTON_PATH: &[&str] = &["WorldView", "NPCRangeWin", "imgBackground", "TeamUpButton"];
pub const TEAM_UP_CONFIRM_PATH: &[&str] = &["WorldView", "TeamUpConfirmationWindow", "TeamUpConfirmationBackground", "TeamUpButton"];

// ── NPC Range Path ──────────────────────────────────────────────────

pub const NPC_RANGE_PATH: &[&str] = &["WorldView", "NPCRangeWin"];

// ── Cancel Chest Roll Path ──────────────────────────────────────────

pub const CANCEL_CHEST_ROLL_PATH: &[&str] = &["WorldView", "Container", "background", "", "CancelButton"];

// ── Missing Area Menu Paths ─────────────────────────────────────────

pub const MISSING_AREA_PATH: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow"];
pub const MISSING_AREA_RETRY_PATH: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "RetryBtn"];

// ── Willcast Card Paths ─────────────────────────────────────────────

pub const WILLCAST_PATH: &[&str] = &["WorldView", "PlanningPhase", "Alignment", "PlanningPhaseSubWindow", "SpellSelection", "Hand", "PetCard", "PetCardWindow"];

// ── Exit Shop Paths ─────────────────────────────────────────────────

pub const EXIT_RECIPE_SHOP_PATH: &[&str] = &["WorldView", "", "Exit"];
pub const EXIT_EQUIPMENT_SHOP_PATH: &[&str] = &["WorldView", "shopGUI", "buyWindow", "exit"];
pub const EXIT_SNACK_SHOP_PATH: &[&str] = &["WorldView", "ShoppingPetSnackWindow", "buyWindow", "exit"];
pub const EXIT_REAGENT_SHOP_PATH: &[&str] = &["WorldView", "ShoppingReagentWindow", "buyWindow", "exit"];

// ── Cancel Quest Menus ──────────────────────────────────────────────

pub const CANCEL_MULTIPLE_QUEST_MENU_PATH: &[&str] = &["WorldView", "NPCServicesWin", "wndDialogMain", "Exit"];
pub const CANCEL_SPELL_VENDOR_PATH: &[&str] = &["WorldView", "NPCTrainingGUI", "TrainingSelection", "Exit"];

// ── Resume Instance Button ──────────────────────────────────────────

pub const RESUME_INSTANCE_PATH: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "ResumeInstanceButton"];

// ── Exit Treasure Card Vendor ───────────────────────────────────────

pub const EXIT_TC_VENDOR_PATH: &[&str] = &["WorldView", "main", "exit"];

// ── Exit Minigame Sigil ─────────────────────────────────────────────

pub const EXIT_MINIGAME_SIGIL_PATH: &[&str] = &["WorldView", "mainwindow", "exit"];

// ── Exit Wysteria Tournament ────────────────────────────────────────

pub const EXIT_WYSTERIA_TOURNAMENT_PATH: &[&str] = &["WorldView", "TournamentRanking", "exit"];

// ── Chat Window ─────────────────────────────────────────────────────

pub const CHAT_WINDOW_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "chatLogContainer", "chatLogInnerContainer", "chatLog"];

// ── Chat Channels ───────────────────────────────────────────────────

pub const MAIN_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "MainContainer", "MainFilterButton"];
pub const GROUP_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "GroupContainer", "GroupFilterButton"];
pub const HOUSE_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "HouseContainer", "HouseFilterButton"];
pub const FRIEND_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "FriendContainer", "FriendFilterButton"];
pub const CHANNEL_ONE_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "Channel1Container", "Channel1FilterButton"];
pub const CHANNEL_TWO_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "Channel2Container", "Channel2FilterButton"];
pub const CHANNEL_THREE_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "Channel3Container", "Channel3FilterButton"];
pub const GUILD_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "Channel4Container", "Channel4FilterButton"];
pub const TEAM_UP_CHAT_CHANNEL_PATH: &[&str] = &["WorldView", "WizardChatBox", "chatContainer", "FilterLayout", "TeamUpContainer", "TeamUpFilterButton"];

// ── Popup Message Path ──────────────────────────────────────────────

pub const POPUP_MSGTEXT_PATH: &[&str] = &["WorldView", "NPCRangeWin", "imgBackground", "NPCRangeTxtMessage"];

// ── Friend Busy / Dungeon Reset Path ────────────────────────────────

pub const FRIEND_IS_BUSY_AND_DUNGEON_RESET_PATH: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "rightButton"];

// ── Friend List / Friend Popup Paths ────────────────────────────────

pub const ADD_REMOVE_FRIEND_PATH: &[&str] = &["WorldView", "windowHUD", "wndCharacter", "ButtonLayout", "btnAddRemoveFriend"];
pub const CONFIRM_SEND_FRIEND_REQUEST_PATH: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];
pub const CONFIRM_ACCEPT_FRIEND_REQUEST_PATH: &[&str] = &["WorldView", "MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];
pub const ENTER_TRUE_FRIEND_CODE_BUTTON_PATH: &[&str] = &["WorldView", "windowHUD", "wndFriendsList", "btnSecureChatSetup"];
pub const TRUE_FRIEND_WINDOW_PATH: &[&str] = &["WorldView", "FriendCodeWindow"];
pub const GENERATE_TRUE_FRIEND_CODE_PATH: &[&str] = &["WorldView", "FriendCodeWindow", "btnGenerateCode"];
pub const CONFIRM_TRUE_FRIEND_CODE_PATH: &[&str] = &["WorldView", "FriendCodeWindow", "ValidateWindow", "btnValidateCode"];
pub const EXIT_TRUE_FRIEND_WINDOW_PATH: &[&str] = &["WorldView", "FriendCodeWindow", "btnExit"];
pub const EXIT_GENERATE_TRUE_FRIEND_WINDOW_PATH: &[&str] = &["WorldView", "", "btnExit"];
pub const TRUE_FRIEND_CODE_TEXT_PATH: &[&str] = &["WorldView", "", "txtRLFCode"];
pub const CLOSE_REAL_FRIEND_LIST_BUTTON_PATH: &[&str] = &["WorldView", "windowHUD", "wndFriendsList", "btnFriendListClose"];

// ── Teleport Mark Recall Path ───────────────────────────────────────

pub const TELEPORT_MARK_RECALL_PATH: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "RecallButton"];
pub const TELEPORT_MARK_RECALL_TIMER_PATH: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "RecallButton", "txtRecallTimer"];

// ── Dungeon Recall Path ─────────────────────────────────────────────

pub const DUNGEON_RECALL_PATH: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "ResumeInstanceButton"];

// ── Open Cantrips Path ──────────────────────────────────────────────

pub const OPEN_CANTRIPS_PATH: &[&str] = &["WorldView", "windowHUD", "compassAndTeleporterButtons", "OpenCantripsButton"];

// ── Exit Dungeon Path ───────────────────────────────────────────────

pub const EXIT_DUNGEON_PATH: &[&str] = &["MessageBoxModalWindow", "messageBoxBG", "messageBoxLayout", "AdjustmentWindow", "Layout", "centerButton"];

// ── Zafaria Class Picture Exit Path ─────────────────────────────────

pub const EXIT_ZAFARIA_CLASS_PICTURE_BUTTON_PATH: &[&str] = &["WorldView", "ClassPicture", "exit"];

// ── Backpack/Equipment Paths ────────────────────────────────────────

pub const BACKPACK_IS_VISIBLE_PATH: &[&str] = &["WorldView", "DeckConfiguration", "InventorySpellbookPage", "EquipmentManager"];
pub const EQUIPMENT_SET_MANAGER_TITLE_PATH: &[&str] = &["WorldView", "DeckConfiguration", "EquipmentManager", "top_scroll", "title"];
pub const INDIVIDUAL_EQUIPMENT_SET_PARENT_PATH: &[&str] = &["WorldView", "DeckConfiguration", "EquipmentManager", ""];
pub const BACKPACK_TITLE_PATH: &[&str] = &["WorldView", "DeckConfiguration", "InventorySpellbookPage", "top_scroll", "title"];

// ── Exit Pet Leveled Up Popup ───────────────────────────────────────

pub const EXIT_PET_LEVELED_UP_BUTTON_PATH: &[&str] = &["WorldView", "PetLevelUpWindow", "wndPetLevelBkg", "btnPetLevelClose"];

// ── Exit Avalon Badge Popup ─────────────────────────────────────────

pub const AVALON_BADGE_EXIT_BUTTON_PATH: &[&str] = &["WorldView", "HelpHousingTips2", "toolbar", "exit"];

// ── Pet Game Paths ──────────────────────────────────────────────────

pub const PET_FEED_WINDOW_VISIBLE_PATH: &[&str] = &["WorldView", "PetGameTracks"];
pub const PET_FEED_WINDOW_CANCEL_BUTTON_PATH: &[&str] = &["WorldView", "PetGameTracks", "btnBack"];
pub const WIZARD_CITY_DANCE_GAME_PATH: &[&str] = &["WorldView", "PetGameTracks", "wndBkgTracks", "wndTracks", "btnTrack0"];
pub const PLAY_DANCE_GAME_BUTTON_PATH: &[&str] = &["WorldView", "PetGameTracks", "btnNext"];
pub const SKIP_PET_GAME_BUTTON_PATH: &[&str] = &["WorldView", "PetGameTracks", "SkipGameButton"];
pub const WON_PET_GAME_REWARDS_WINDOW_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards"];
pub const SKIPPED_PET_GAME_REWARDS_WINDOW_PATH: &[&str] = &["WorldView", "", "PetGameRewards"];
pub const WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "btnNext"];
pub const SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH: &[&str] = &["WorldView", "", "PetGameRewards", "btnNext"];
pub const SKIPPED_FIRST_PET_SNACK_PATH: &[&str] = &["WorldView", "", "PetGameRewards", "wndBkgBottom", "wndCards", "chkSnackCard0"];
pub const WON_FIRST_PET_SNACK_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "wndBkgBottom", "wndCards", "chkSnackCard0"];
pub const WON_PET_GAME_PLAY_AGAIN_BUTTON_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "btnPlayAgain"];
pub const WON_FINISH_PET_BUTTON_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameRewards", "btnBack"];
pub const SKIPPED_FINISH_PET_BUTTON_PATH: &[&str] = &["WorldView", "", "PetGameRewards", "btnBack"];
pub const SKIPPED_PET_GAME_PLAY_AGAIN_BUTTON_PATH: &[&str] = &["WorldView", "", "PetGameRewards", "btnPlayAgain"];
pub const PET_FEED_WINDOW_ENERGY_COST_TEXTBOX_PATH: &[&str] = &["WorldView", "PetGameTracks", "wndBkgEnergy", "txtEnergyCost"];
pub const PET_FEED_WINDOW_YOUR_ENERGY_TEXTBOX_PATH: &[&str] = &["WorldView", "PetGameTracks", "wndBkgEnergy", "txtYourEnergy"];
pub const SKIPPED_PET_LEVELED_UP_WINDOW_PATH: &[&str] = &["WorldView", "", "PetLevelUpWindow"];
pub const WON_PET_LEVELED_UP_WINDOW_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetLevelUpWindow"];
pub const WON_PET_LEVELED_UP_ANNOUNCEMENT_TXT_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetLevelUpWindow", "wndPetLevelBkg", "txtAnnounceText"];
pub const SKIPPED_PET_LEVELED_UP_ANNOUNCEMENT_TXT_PATH: &[&str] = &["WorldView", "", "PetLevelUpWindow", "wndPetLevelBkg", "txtTitle"];
pub const WON_PET_LEVELED_UP_TALENT_TXT_PATH: &[&str] = &["WorldView", "", "PetLevelUpWindow", "wndPetLevelBkg", "", "txtName"];
pub const SKIPPED_PET_LEVELED_UP_TALENT_TXT_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetLevelUpWindow", "wndPetLevelBkg", "", "powerGained", "txtName"];
pub const EXIT_WON_PET_LEVELED_UP_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetLevelUpWindow", "wndPetLevelBkg", "btnPetLevelClose"];
pub const EXIT_SKIPPED_PET_LEVELED_UP_PATH: &[&str] = &["WorldView", "", "PetLevelUpWindow", "wndPetLevelBkg", "btnPetLevelClose"];
pub const DANCE_GAME_ACTION_TEXTBOX_PATH: &[&str] = &["WorldView", "PetGameSplash", "", "PetGameDance", "wndControls", "wndActionBkg", "txtAction"];

// ── Character Screen Energy Amount ──────────────────────────────────

pub const ENERGY_AMOUNT_PATH: &[&str] = &["WorldView", "DeckConfiguration", "", "ControlSprite", "wndEnergyFrame", "Layout", "txtEnergy"];

// ── Questbook Quest Paths ───────────────────────────────────────────

pub const QUEST_BUTTONS_PARENT_PATH: &[&str] = &["WorldView", "DeckConfiguration", "wndQuestList"];
pub const QUEST_TWO_BUTTON_PATH: &[&str] = &["WorldView", "DeckConfiguration", "wndQuestList", "wndQuestInfo1"];
pub const ALL_QUESTS_SORT_BUTTON_PATH: &[&str] = &["WorldView", "DeckConfiguration", "wndQuestList", "QuestLogAllButton"];

// ── Deprecated/Legacy Aliases (to minimize breakage) ──────────────

pub const SPIRAL_DOOR_RESET: &[&str] = SPIRAL_DOOR_RESET_PATH;
pub const SPIRAL_DOOR_CYCLE: &[&str] = SPIRAL_DOOR_CYCLE_PATH;
pub const SPIRAL_DOOR_TELEPORT: &[&str] = SPIRAL_DOOR_TELEPORT_PATH;
pub const SPIRAL_DOOR_WORLD: &[&str] = SPIRAL_DOOR_WORLD_PATH;
pub const SPIRAL_DOOR_SELECTED: &[&str] = SPIRAL_DOOR_SELECTED_PATH;
pub const SPIRAL_DOOR: &[&str] = SPIRAL_DOOR_PATH;
pub const SPIRAL_DOOR_TITLE: &[&str] = SPIRAL_DOOR_TITLE_PATH;
pub const SPIRAL_DOOR_EXIT: &[&str] = SPIRAL_DOOR_EXIT_PATH;
pub const POTION_SHOP_BASE: &[&str] = POTION_SHOP_BASE_PATH;
pub const POTION_BUY: &[&str] = POTION_BUY_PATH;
pub const POTION_FILL_ALL: &[&str] = POTION_FILL_ALL_PATH;
pub const POTION_EXIT: &[&str] = POTION_EXIT_PATH;
pub const POTION_USAGE: &[&str] = POTION_USAGE_PATH;
pub const CHECK_SPELLBOOK_OPEN: &[&str] = CHECK_SPELLBOOK_OPEN_PATH;
pub const QUIT_BUTTON: &[&str] = QUIT_BUTTON_PATH;
pub const CLOSE_SPELLBOOK: &[&str] = CLOSE_SPELLBOOK_PATH;
pub const PLAY_BUTTON: &[&str] = PLAY_BUTTON_PATH;
pub const DUNGEON_WARNING: &[&str] = DUNGEON_WARNING_PATH;
pub const ADVANCE_DIALOG: &[&str] = ADVANCE_DIALOG_PATH;
pub const DECLINE_QUEST: &[&str] = DECLINE_QUEST_PATH;
pub const DIALOG_TEXT: &[&str] = DIALOG_TEXT_PATH;
pub const QUEST_NAME: &[&str] = QUEST_NAME_PATH;
pub const POPUP_TITLE: &[&str] = POPUP_TITLE_PATH;
pub const POPUP_MSGTEXT: &[&str] = POPUP_MSGTEXT_PATH;
pub const TEAM_UP_BUTTON: &[&str] = TEAM_UP_BUTTON_PATH;
pub const TEAM_UP_CONFIRM: &[&str] = TEAM_UP_CONFIRM_PATH;
pub const NPC_RANGE: &[&str] = NPC_RANGE_PATH;
pub const CANCEL_CHEST_ROLL: &[&str] = CANCEL_CHEST_ROLL_PATH;
pub const MISSING_AREA: &[&str] = MISSING_AREA_PATH;
pub const MISSING_AREA_RETRY: &[&str] = MISSING_AREA_RETRY_PATH;
pub const WILLCAST: &[&str] = WILLCAST_PATH;
pub const EXIT_RECIPE_SHOP: &[&str] = EXIT_RECIPE_SHOP_PATH;
pub const EXIT_EQUIPMENT_SHOP: &[&str] = EXIT_EQUIPMENT_SHOP_PATH;
pub const EXIT_SNACK_SHOP: &[&str] = EXIT_SNACK_SHOP_PATH;
pub const EXIT_REAGENT_SHOP: &[&str] = EXIT_REAGENT_SHOP_PATH;
pub const EXIT_TC_VENDOR: &[&str] = EXIT_TC_VENDOR_PATH;
pub const EXIT_MINIGAME_SIGIL: &[&str] = EXIT_MINIGAME_SIGIL_PATH;
pub const CANCEL_MULTIPLE_QUEST_MENU: &[&str] = CANCEL_MULTIPLE_QUEST_MENU_PATH;
pub const CANCEL_SPELL_VENDOR: &[&str] = CANCEL_SPELL_VENDOR_PATH;
pub const RESUME_INSTANCE: &[&str] = RESUME_INSTANCE_PATH;
pub const EXIT_DUNGEON: &[&str] = EXIT_DUNGEON_PATH;
pub const FRIEND_IS_BUSY_AND_DUNGEON_RESET: &[&str] = FRIEND_IS_BUSY_AND_DUNGEON_RESET_PATH;
pub const ADD_REMOVE_FRIEND: &[&str] = ADD_REMOVE_FRIEND_PATH;
pub const CONFIRM_SEND_FRIEND_REQUEST: &[&str] = CONFIRM_SEND_FRIEND_REQUEST_PATH;
pub const TELEPORT_MARK_RECALL: &[&str] = TELEPORT_MARK_RECALL_PATH;
pub const TELEPORT_MARK_RECALL_TIMER: &[&str] = TELEPORT_MARK_RECALL_TIMER_PATH;
pub const CHAT_WINDOW: &[&str] = CHAT_WINDOW_PATH;
pub const BACKPACK_IS_VISIBLE: &[&str] = BACKPACK_IS_VISIBLE_PATH;
pub const EQUIPMENT_SET_MANAGER_TITLE: &[&str] = EQUIPMENT_SET_MANAGER_TITLE_PATH;
pub const BACKPACK_TITLE: &[&str] = BACKPACK_TITLE_PATH;
pub const EXIT_ZAFARIA_CLASS_PICTURE: &[&str] = EXIT_ZAFARIA_CLASS_PICTURE_BUTTON_PATH;
pub const EXIT_PET_LEVELED_UP: &[&str] = EXIT_PET_LEVELED_UP_BUTTON_PATH;
pub const AVALON_BADGE_EXIT: &[&str] = AVALON_BADGE_EXIT_BUTTON_PATH;
pub const PET_FEED_WINDOW_VISIBLE: &[&str] = PET_FEED_WINDOW_VISIBLE_PATH;
pub const PET_FEED_WINDOW_CANCEL_BUTTON: &[&str] = PET_FEED_WINDOW_CANCEL_BUTTON_PATH;
pub const PLAY_DANCE_GAME_BUTTON: &[&str] = PLAY_DANCE_GAME_BUTTON_PATH;
pub const SKIP_PET_GAME_BUTTON: &[&str] = SKIP_PET_GAME_BUTTON_PATH;
pub const WON_PET_GAME_REWARDS_WINDOW: &[&str] = WON_PET_GAME_REWARDS_WINDOW_PATH;
pub const SKIPPED_PET_GAME_REWARDS_WINDOW: &[&str] = SKIPPED_PET_GAME_REWARDS_WINDOW_PATH;
pub const WON_PET_GAME_CONTINUE_BUTTON: &[&str] = WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH;
pub const SKIPPED_PET_GAME_CONTINUE_BUTTON: &[&str] = SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH;
pub const WON_PET_GAME_PLAY_AGAIN_BUTTON: &[&str] = WON_PET_GAME_PLAY_AGAIN_BUTTON_PATH;
pub const WON_FINISH_PET_BUTTON: &[&str] = WON_FINISH_PET_BUTTON_PATH;
pub const SKIPPED_FINISH_PET_BUTTON: &[&str] = SKIPPED_FINISH_PET_BUTTON_PATH;
pub const PET_FEED_ENERGY_COST: &[&str] = PET_FEED_WINDOW_ENERGY_COST_TEXTBOX_PATH;
pub const PET_FEED_YOUR_ENERGY: &[&str] = PET_FEED_WINDOW_YOUR_ENERGY_TEXTBOX_PATH;
pub const DANCE_GAME_ACTION_TEXT: &[&str] = DANCE_GAME_ACTION_TEXTBOX_PATH;
pub const ENERGY_AMOUNT: &[&str] = ENERGY_AMOUNT_PATH;
pub const SKIPPED_FIRST_PET_SNACK: &[&str] = SKIPPED_FIRST_PET_SNACK_PATH;
pub const WON_FIRST_PET_SNACK: &[&str] = WON_FIRST_PET_SNACK_PATH;
pub const SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON: &[&str] = SKIPPED_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH;
pub const WON_PET_GAME_CONTINUE_AND_FEED_BUTTON: &[&str] = WON_PET_GAME_CONTINUE_AND_FEED_BUTTON_PATH;
pub const SKIPPED_PET_LEVELED_UP_WINDOW: &[&str] = SKIPPED_PET_LEVELED_UP_WINDOW_PATH;
pub const WON_PET_LEVELED_UP_WINDOW: &[&str] = WON_PET_LEVELED_UP_WINDOW_PATH;
pub const EXIT_SKIPPED_PET_LEVELED_UP: &[&str] = EXIT_SKIPPED_PET_LEVELED_UP_PATH;
pub const EXIT_WON_PET_LEVELED_UP: &[&str] = EXIT_WON_PET_LEVELED_UP_PATH;
pub const WIZARD_CITY_DANCE_GAME: &[&str] = WIZARD_CITY_DANCE_GAME_PATH;
pub const DANCE_GAME_ACTION_TEXTBOX: &[&str] = DANCE_GAME_ACTION_TEXTBOX_PATH;
pub const CONFIRM_ACCEPT_FRIEND_REQUEST: &[&str] = CONFIRM_ACCEPT_FRIEND_REQUEST_PATH;
pub const CLOSE_REAL_FRIEND_LIST_BUTTON: &[&str] = CLOSE_REAL_FRIEND_LIST_BUTTON_PATH;
pub const EXIT_WYSTERIA_TOURNAMENT: &[&str] = EXIT_WYSTERIA_TOURNAMENT_PATH;

// Marker for logic faithfulness.
// ADDED logic: Verified 1:1 against paths.py.
// Unified naming convention using _PATH suffix.
