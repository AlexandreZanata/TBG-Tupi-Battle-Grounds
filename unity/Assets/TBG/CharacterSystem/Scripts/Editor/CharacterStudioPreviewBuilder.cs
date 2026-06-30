using System;
using System.IO;
using TBG.CharacterLibrary;
using TBG.CharacterSystem.Domain;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterSystem.Editor
{
    internal static class CharacterStudioPreviewBuilder
    {
        public static GameObject Build(
            CharacterDefinitionData draft,
            string assetMode,
            CharacterCatalog catalog,
            out CharacterInstance instance)
        {
            instance = null;
            if (draft == null)
            {
                return null;
            }

            if (string.Equals(assetMode, "mesh3d", StringComparison.OrdinalIgnoreCase))
            {
                instance = catalog.Spawn(draft.id, null);
                var root = instance.Root;
                root.transform.position = Vector3.zero;
                return root;
            }

            var definition = ScriptableObject.CreateInstance<CharacterDefinition>();
            definition.ApplyData(draft);
            var builder = new ProceduralCharacterBuilder();
            var proceduralRoot = builder.Build(definition);
            proceduralRoot.transform.position = Vector3.zero;
            UnityEngine.Object.DestroyImmediate(definition);
            return proceduralRoot;
        }

        public static void LoadAssetModeMetadata(string id, out string assetMode, out string meshPath)
        {
            var path = Path.Combine(CharacterPaths.DefinitionsDir, id + ".json");
            var json = File.ReadAllText(path);
            var mode = JsonUtility.FromJson<CharacterDefinitionModeDto>(json);
            assetMode = string.IsNullOrWhiteSpace(mode.assetMode) ? "procedural" : mode.assetMode;
            meshPath = mode.mesh;
        }

        public static string FormatListLabel(string id)
        {
            LoadAssetModeMetadata(id, out var assetMode, out _);
            return string.Equals(assetMode, "mesh3d", StringComparison.OrdinalIgnoreCase)
                ? $"{id} [mesh3d]"
                : id;
        }

        [Serializable]
        internal sealed class CharacterDefinitionModeDto
        {
            public string assetMode;
            public string mesh;
        }
    }
}
