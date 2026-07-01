using System;
using System.IO;

namespace TBG.CharacterLibrary
{
    public static class AssemblyMeshPath
    {
        public const string ExportRoot = "content/characters/library/exports";

        public static bool HasRecipe(CharacterDefinitionJsonRecord record)
        {
            return record?.assemblyRecipe != null
                && !string.IsNullOrWhiteSpace(record.assemblyRecipe.unitId);
        }

        public static string Resolve(CharacterDefinitionJsonRecord record)
        {
            if (HasRecipe(record))
            {
                return Path.Combine(ExportRoot, record.assemblyRecipe.unitId + ".glb")
                    .Replace('\\', '/');
            }

            return record?.mesh;
        }
    }
}
