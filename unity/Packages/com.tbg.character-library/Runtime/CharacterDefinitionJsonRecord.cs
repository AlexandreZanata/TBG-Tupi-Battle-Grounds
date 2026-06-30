using System;

namespace TBG.CharacterLibrary
{
    [Serializable]
    public sealed class CharacterTexturesRecord
    {
        public string albedo;
        public string normal;
        public string orm;
        public string emissive;
    }

    [Serializable]
    public sealed class CharacterDefinitionJsonRecord
    {
        public string id;
        public string characterName;
        public string archetype;
        public string faction;
        public string assetMode;
        public string mesh;
        public CharacterTexturesRecord textures;
        public string materialProfile;
        public string behaviorProfile;
        public string animatorController;
        public float heightScale = 1f;
        public float bodyWidthScale = 1f;
        public float headScale = 1f;
        public float limbLengthScale = 1f;
        public string primaryColor = "#FFFFFF";
        public string secondaryColor = "#CCCCCC";
        public string[] accessories;
        public int evolutionStage;
    }
}
