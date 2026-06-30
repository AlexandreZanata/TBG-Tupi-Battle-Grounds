using System;
using System.IO;
using TBG.CharacterSystem.Domain;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class Mesh3DCharacterFactory
    {
        public GameObject Create(LoadedCharacterDefinition definition, Transform parent)
        {
            if (string.IsNullOrWhiteSpace(definition.MeshPath))
            {
                throw new InvalidOperationException("mesh3d assetMode requires mesh path.");
            }

            if (!definition.MeshPath.EndsWith(".glb", StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException("mesh path must be a .glb file.");
            }

            var repoRoot = CharacterPaths.GetRepoRoot();
            var absoluteMeshPath = Path.GetFullPath(Path.Combine(repoRoot, definition.MeshPath));
            if (!File.Exists(absoluteMeshPath))
            {
                throw new FileNotFoundException("Mesh file not found.", absoluteMeshPath);
            }

            // ponytail: glTFast import wired in Phase 02 §8 (Mesh3DCharacterFactory.Load)
            var root = new GameObject(definition.CharacterName);
            root.transform.SetParent(parent, false);
            return root;
        }
    }
}
