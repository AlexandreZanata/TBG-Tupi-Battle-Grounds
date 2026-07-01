using System;

namespace TBG.CharacterLibrary
{
    public sealed class GameLodSettings
    {
        public int TriTarget { get; init; } = 8000;
        public int TextureSize { get; init; } = 1024;

        public static GameLodSettings FromRecord(GameLodRecord record)
        {
            if (record == null)
            {
                return new GameLodSettings();
            }

            return new GameLodSettings
            {
                TriTarget = record.triTarget > 0 ? record.triTarget : 8000,
                TextureSize = record.textureSize > 0 ? record.textureSize : 1024
            };
        }
    }

    public sealed class LoadedCharacterDefinition
    {
        public string Id { get; init; }
        public string CharacterName { get; init; }
        public CharacterAssetMode AssetMode { get; init; }
        public string MeshPath { get; init; }
        public string MeshShowcasePath { get; init; }
        public string AnimationSetId { get; init; }
        public GameLodSettings GameLod { get; init; }
        public string MaterialProfileId { get; init; }
        public string BehaviorProfileId { get; init; }
        public string AnimatorControllerPath { get; init; }
        public CharacterTexturePaths Textures { get; init; }
        public CharacterDefinitionData ProceduralData { get; init; }

        public static LoadedCharacterDefinition FromRecord(CharacterDefinitionJsonRecord record)
        {
            if (record == null || string.IsNullOrWhiteSpace(record.id))
            {
                throw new ArgumentException("Character definition id is required.");
            }

            return new LoadedCharacterDefinition
            {
                Id = record.id,
                CharacterName = record.characterName,
                AssetMode = ParseAssetMode(record.assetMode),
                MeshPath = AssemblyMeshPath.Resolve(record),
                MeshShowcasePath = record.meshShowcase,
                AnimationSetId = record.animationSet,
                GameLod = GameLodSettings.FromRecord(record.gameLod),
                MaterialProfileId = record.materialProfile,
                BehaviorProfileId = record.behaviorProfile,
                AnimatorControllerPath = record.animatorController,
                Textures = CharacterTexturePaths.FromRecord(record.textures),
                ProceduralData = ToProceduralData(record)
            };
        }

        private static CharacterAssetMode ParseAssetMode(string value)
        {
            if (string.Equals(value, "mesh3d", StringComparison.OrdinalIgnoreCase))
            {
                return CharacterAssetMode.Mesh3D;
            }

            return CharacterAssetMode.Procedural;
        }

        private static CharacterDefinitionData ToProceduralData(CharacterDefinitionJsonRecord record)
        {
            return new CharacterDefinitionData
            {
                id = record.id,
                characterName = record.characterName,
                archetype = ParseEnum<Archetype>(record.archetype),
                faction = ParseEnum<Faction>(record.faction),
                heightScale = record.heightScale,
                bodyWidthScale = record.bodyWidthScale,
                headScale = record.headScale,
                limbLengthScale = record.limbLengthScale,
                primaryColor = record.primaryColor,
                secondaryColor = record.secondaryColor,
                accessories = ParseAccessories(record.accessories),
                evolutionStage = record.evolutionStage
            };
        }

        private static T ParseEnum<T>(string value) where T : struct
        {
            return Enum.TryParse(value, out T parsed) ? parsed : default;
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
