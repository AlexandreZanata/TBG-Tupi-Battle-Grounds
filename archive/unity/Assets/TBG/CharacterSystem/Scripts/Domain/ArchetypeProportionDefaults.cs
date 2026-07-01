using UnityEngine;

namespace TBG.CharacterSystem.Domain
{
    public static class ArchetypeProportionDefaults
    {
        public static void Apply(Archetype archetype, CharacterDefinitionData data)
        {
            switch (archetype)
            {
                case Archetype.Tank:
                    data.heightScale = 1.2f;
                    data.bodyWidthScale = 1.8f;
                    data.headScale = 0.8f;
                    data.limbLengthScale = 0.7f;
                    break;
                case Archetype.DPS:
                    data.heightScale = 1.0f;
                    data.bodyWidthScale = 0.8f;
                    data.headScale = 0.9f;
                    data.limbLengthScale = 1.1f;
                    break;
                case Archetype.Support:
                    data.heightScale = 0.9f;
                    data.bodyWidthScale = 0.7f;
                    data.headScale = 1.2f;
                    data.limbLengthScale = 0.8f;
                    break;
                case Archetype.Ranged:
                    data.heightScale = 1.1f;
                    data.bodyWidthScale = 0.6f;
                    data.headScale = 0.8f;
                    data.limbLengthScale = 1.3f;
                    break;
                default:
                    data.heightScale = 1.0f;
                    data.bodyWidthScale = 0.7f;
                    data.headScale = 1.1f;
                    data.limbLengthScale = 0.9f;
                    break;
            }
        }
    }
}
