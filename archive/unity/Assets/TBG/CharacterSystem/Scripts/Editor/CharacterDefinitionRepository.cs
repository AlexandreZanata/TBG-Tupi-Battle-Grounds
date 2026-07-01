using System;
using System.Collections.Generic;
using System.IO;
using TBG.CharacterSystem.Domain;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterSystem.Editor
{
    public static class CharacterDefinitionRepository
    {
        public static IReadOnlyList<string> ListIds()
        {
            var ids = new List<string>();
            if (!Directory.Exists(CharacterPaths.DefinitionsDir))
            {
                return ids;
            }

            foreach (var file in Directory.GetFiles(CharacterPaths.DefinitionsDir, "*.json"))
            {
                var name = Path.GetFileNameWithoutExtension(file);
                if (!name.StartsWith("_"))
                {
                    ids.Add(name);
                }
            }

            ids.Sort();
            return ids;
        }

        public static CharacterDefinitionData Load(string id)
        {
            var path = Path.Combine(CharacterPaths.DefinitionsDir, id + ".json");
            var json = File.ReadAllText(path);
            var dto = JsonUtility.FromJson<CharacterDefinitionJsonDto>(json);
            return CharacterDefinitionJsonMapper.FromDto(dto);
        }

        public static void Save(CharacterDefinitionData data)
        {
            Directory.CreateDirectory(CharacterPaths.DefinitionsDir);
            var path = Path.Combine(CharacterPaths.DefinitionsDir, data.id + ".json");
            var dto = CharacterDefinitionJsonMapper.ToDto(data);
            File.WriteAllText(path, JsonUtility.ToJson(dto, true));
        }

        public static void Delete(string id)
        {
            DeleteIfExists(Path.Combine(CharacterPaths.DefinitionsDir, id + ".json"));
            DeleteIfExists(Path.Combine(CharacterPaths.CardsDir, id + ".png"));
            DeleteIfExists(Path.Combine(CharacterPaths.PreviewsDir, id + ".png"));
        }

        private static void DeleteIfExists(string path)
        {
            if (File.Exists(path))
            {
                File.Delete(path);
            }
        }
    }
}
