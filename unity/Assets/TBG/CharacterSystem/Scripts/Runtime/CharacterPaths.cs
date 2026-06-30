using System.IO;
using UnityEngine;

namespace TBG.CharacterSystem.Runtime
{
    public static class CharacterPaths
    {
        public static string GetRepoRoot()
        {
            var dataPath = Application.dataPath;
            return Path.GetFullPath(Path.Combine(dataPath, "..", ".."));
        }

        public static string DefinitionsDir => Path.Combine(GetContentRoot(), "definitions");
        public static string CardsDir => Path.Combine(GetContentRoot(), "cards");
        public static string PreviewsDir => Path.Combine(GetContentRoot(), "previews");
        public static string TexturesDir => Path.Combine(GetContentRoot(), "textures");
        public static string MaterialsDir => Path.Combine(GetContentRoot(), "materials");

        public static string GetContentRoot()
        {
            return Path.Combine(GetRepoRoot(), "content", "characters");
        }
    }
}
