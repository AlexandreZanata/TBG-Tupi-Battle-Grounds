using System;
using TBG.CharacterSystem.Domain;

namespace TBG.CharacterSystem.Editor
{
    [Serializable]
    public class CharacterDefinitionJsonDto
    {
        public string id;
        public string characterName;
        public string archetype;
        public string faction;
        public float heightScale;
        public float bodyWidthScale;
        public float headScale;
        public float limbLengthScale;
        public string primaryColor;
        public string secondaryColor;
        public string[] accessories;
        public int evolutionStage;
    }

    public static class CharacterDefinitionJsonMapper
    {
        public static CharacterDefinitionData FromDto(CharacterDefinitionJsonDto dto)
        {
            return new CharacterDefinitionData
            {
                id = dto.id,
                characterName = dto.characterName,
                archetype = ParseEnum<Archetype>(dto.archetype),
                faction = ParseEnum<Faction>(dto.faction),
                heightScale = dto.heightScale,
                bodyWidthScale = dto.bodyWidthScale,
                headScale = dto.headScale,
                limbLengthScale = dto.limbLengthScale,
                primaryColor = dto.primaryColor,
                secondaryColor = dto.secondaryColor,
                accessories = ParseAccessories(dto.accessories),
                evolutionStage = dto.evolutionStage
            };
        }

        public static CharacterDefinitionJsonDto ToDto(CharacterDefinitionData data)
        {
            var accessories = new string[data.accessories.Length];
            for (var i = 0; i < data.accessories.Length; i++)
            {
                accessories[i] = data.accessories[i].ToString();
            }

            return new CharacterDefinitionJsonDto
            {
                id = data.id,
                characterName = data.characterName,
                archetype = data.archetype.ToString(),
                faction = data.faction.ToString(),
                heightScale = data.heightScale,
                bodyWidthScale = data.bodyWidthScale,
                headScale = data.headScale,
                limbLengthScale = data.limbLengthScale,
                primaryColor = data.primaryColor,
                secondaryColor = data.secondaryColor,
                accessories = accessories,
                evolutionStage = data.evolutionStage
            };
        }

        private static T ParseEnum<T>(string value) where T : struct
        {
            return Enum.TryParse<T>(value, out var parsed) ? parsed : default;
        }

        private static AccessoryType[] ParseAccessories(string[] values)
        {
            if (values == null || values.Length == 0)
            {
                return Array.Empty<AccessoryType>();
            }

            var result = new AccessoryType[values.Length];
            for (var i = 0; i < values.Length; i++)
            {
                Enum.TryParse(values[i], out result[i]);
            }

            return result;
        }
    }
}
