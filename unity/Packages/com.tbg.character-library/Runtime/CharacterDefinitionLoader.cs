using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class CharacterDefinitionLoader
    {
        public LoadedCharacterDefinition Load(string id)
        {
            var path = Path.Combine(CharacterPaths.DefinitionsDir, id + ".json");
            if (!File.Exists(path))
            {
                throw new FileNotFoundException($"Character definition not found: {id}", path);
            }

            var json = File.ReadAllText(path);
            var record = JsonUtility.FromJson<CharacterDefinitionJsonRecord>(json);
            CharacterDefinitionValidator.ValidateOrThrow(record);
            return LoadedCharacterDefinition.FromRecord(record);
        }
    }
}
