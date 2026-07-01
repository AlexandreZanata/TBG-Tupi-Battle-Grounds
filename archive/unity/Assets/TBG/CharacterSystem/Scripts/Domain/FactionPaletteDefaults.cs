namespace TBG.CharacterSystem.Domain
{
    public static class FactionPaletteDefaults
    {
        public static (string primary, string secondary) GetColors(Faction faction)
        {
            switch (faction)
            {
                case Faction.Humans: return ("#8B7355", "#C4A882");
                case Faction.Undead: return ("#4A5D23", "#7D8B55");
                case Faction.Machines: return ("#4A6FA5", "#B0C4DE");
                case Faction.Plants: return ("#2D5A27", "#6B8E23");
                default: return ("#FFFFFF", "#CCCCCC");
            }
        }
    }
}
