using System;

namespace TBG.CharacterLibrary
{
    [Serializable]
    public sealed class MaterialProfilePropertiesRecord
    {
        public float _Metallic = 0.1f;
        public float _Smoothness = 0.5f;
        public float _OcclusionStrength = 1f;
        public float _EmissionStrength;
    }

    [Serializable]
    public sealed class MaterialProfileJsonRecord
    {
        public string id;
        public string faction;
        public string shader;
        public string graph;
        public MaterialProfilePropertiesRecord properties;
        public string[] textureSlots;
    }
}
