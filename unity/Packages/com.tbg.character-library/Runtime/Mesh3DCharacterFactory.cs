using System;
using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class Mesh3DCharacterFactory
    {
        private readonly MaterialProfileLoader _materialLoader;
        private readonly BehaviorProfileLoader _behaviorLoader;

        public Mesh3DCharacterFactory()
            : this(new MaterialProfileLoader(), new BehaviorProfileLoader())
        {
        }

        public Mesh3DCharacterFactory(
            MaterialProfileLoader materialLoader,
            BehaviorProfileLoader behaviorLoader)
        {
            _materialLoader = materialLoader ?? new MaterialProfileLoader();
            _behaviorLoader = behaviorLoader ?? new BehaviorProfileLoader();
        }

        public GameObject Create(LoadedCharacterDefinition definition, Transform parent)
        {
            ValidateMeshPath(definition);
            var absoluteMeshPath = ResolveAbsolutePath(definition.MeshPath);
            return Load(absoluteMeshPath, definition, parent);
        }

        public GameObject Load(string absoluteGlbPath, LoadedCharacterDefinition definition, Transform parent)
        {
            if (string.IsNullOrWhiteSpace(absoluteGlbPath))
            {
                throw new ArgumentException("glb path is required.", nameof(absoluteGlbPath));
            }

            if (!File.Exists(absoluteGlbPath))
            {
                throw new FileNotFoundException("Mesh file not found.", absoluteGlbPath);
            }

            var root = new GameObject(definition.CharacterName);
            root.transform.SetParent(parent, false);

            GltfMeshLoader.Instantiate(absoluteGlbPath, root.transform);
            Mesh3DPresentationScaler.Apply(root.transform, definition.ProceduralData);
            Mesh3DMaterialApplier.Apply(root, definition, _materialLoader);

            var behaviorProfile = ResolveBehaviorProfile(definition);
            Mesh3DAnimatorSetup.Attach(root, definition, behaviorProfile);

            return root;
        }

        private static void ValidateMeshPath(LoadedCharacterDefinition definition)
        {
            if (string.IsNullOrWhiteSpace(definition.MeshPath))
            {
                throw new InvalidOperationException("mesh3d assetMode requires mesh path.");
            }

            if (!definition.MeshPath.EndsWith(".glb", StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException("mesh path must be a .glb file.");
            }
        }

        private static string ResolveAbsolutePath(string repoRelativePath)
        {
            return Path.GetFullPath(Path.Combine(CharacterPaths.GetRepoRoot(), repoRelativePath));
        }

        private BehaviorProfile ResolveBehaviorProfile(LoadedCharacterDefinition definition)
        {
            if (string.IsNullOrWhiteSpace(definition.BehaviorProfileId))
            {
                return BehaviorProfile.CreateDefault();
            }

            return _behaviorLoader.Load(definition.BehaviorProfileId);
        }
    }
}
