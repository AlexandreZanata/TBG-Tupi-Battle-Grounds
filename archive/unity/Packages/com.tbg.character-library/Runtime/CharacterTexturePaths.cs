namespace TBG.CharacterLibrary
{
    public sealed class CharacterTexturePaths
    {
        public string Albedo { get; init; }
        public string Normal { get; init; }
        public string Orm { get; init; }
        public string Emissive { get; init; }

        public static CharacterTexturePaths FromRecord(CharacterTexturesRecord record)
        {
            if (record == null)
            {
                return null;
            }

            return new CharacterTexturePaths
            {
                Albedo = record.albedo,
                Normal = record.normal,
                Orm = record.orm,
                Emissive = record.emissive
            };
        }
    }
}
