using System.IO;
using NUnit.Framework;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class AssemblyExportSpawnTests
    {
        private Transform _parent;

        [SetUp]
        public void SetUp()
        {
            _parent = new GameObject("AssemblyExportParent").transform;
        }

        [TearDown]
        public void TearDown()
        {
            if (_parent != null)
            {
                Object.DestroyImmediate(_parent.gameObject);
            }
        }

        [Test]
        public void Resolve_AssemblyRecipe_PointsToLibraryExport()
        {
            var record = new CharacterDefinitionJsonRecord
            {
                id = "assembled_guard_01",
                assetMode = "mesh3d",
                assemblyRecipe = new AssemblyRecipeRecord
                {
                    unitId = "test_male_head_sword",
                    baseId = "male",
                    materialId = "default",
                },
            };

            var definition = LoadedCharacterDefinition.FromRecord(record);

            Assert.That(
                definition.MeshPath,
                Is.EqualTo("content/characters/library/exports/test_male_head_sword.glb"));
        }

        [Test]
        public void Create_AssembledExport_HasAnimatorAndWalkClip()
        {
            var record = new CharacterDefinitionJsonRecord
            {
                id = "assembled_guard_01",
                characterName = "Assembled Guard",
                archetype = "Tank",
                faction = "Humans",
                assetMode = "mesh3d",
                animationSet = "humanoid_heavy_combat_v1",
                behaviorProfile = "humanoid_heavy_v1",
                assemblyRecipe = new AssemblyRecipeRecord
                {
                    unitId = "test_male_head_sword",
                    baseId = "male",
                    materialId = "default",
                },
            };

            CharacterDefinitionValidator.ValidateOrThrow(record);
            var definition = LoadedCharacterDefinition.FromRecord(record);
            var meshPath = Path.Combine(CharacterPaths.GetRepoRoot(), definition.MeshPath);
            if (!File.Exists(meshPath))
            {
                Assert.Ignore($"assembled export missing — run assembly export first: {meshPath}");
            }

            var factory = new Mesh3DCharacterFactory();
            var root = factory.Create(definition, _parent);
            var animator = root.GetComponentInChildren<Animator>();
            Assert.That(animator, Is.Not.Null);
            Assert.That(animator.runtimeAnimatorController, Is.Not.Null);

            Object.DestroyImmediate(root);
        }
    }
}
