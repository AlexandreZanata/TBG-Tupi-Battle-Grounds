using System.IO;
using NUnit.Framework;
using TBG.CharacterLibrary;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class PilotRosterEditModeTests
    {
        private Transform _parent;

        [SetUp]
        public void SetUp()
        {
            _parent = new GameObject("PilotRosterParent").transform;
        }

        [TearDown]
        public void TearDown()
        {
            if (_parent != null)
            {
                Object.DestroyImmediate(_parent.gameObject);
            }
        }

        [TestCase("demo_guard_01", "AttackHeavy")]
        [TestCase("demo_striker_01", "AttackFast")]
        [TestCase("demo_scout_01", "AttackRanged")]
        public void Spawn_Mesh3dDemo_LoadsMeshAndPlaysBehaviors(string pilotId, string attackState)
        {
            var catalog = new CharacterCatalog();
            var instance = catalog.Spawn(pilotId, _parent);

            Assert.That(instance.Id, Is.EqualTo(pilotId));
            Assert.That(instance.Root.transform.parent, Is.EqualTo(_parent));
            Assert.That(instance.Root.GetComponentsInChildren<MeshFilter>().Length, Is.GreaterThan(0));
            Assert.That(instance.Root.GetComponentInChildren<Animator>(), Is.Not.Null);

            var behavior = instance.Behavior as BehaviorPlayer;
            Assert.That(behavior, Is.Not.Null);
            Assert.That(behavior.GetStateName(CharacterAction.Attack), Is.EqualTo(attackState));

            Assert.DoesNotThrow(() => instance.PlayAction(CharacterAction.Attack));
            Assert.DoesNotThrow(() => instance.PlayAction(CharacterAction.Death));

            var meshPath = Path.Combine(CharacterPaths.GetRepoRoot(), $"content/characters/meshes/{pilotId}.glb");
            Assert.That(File.Exists(meshPath), Is.True, $"missing demo glb: {meshPath}");

            catalog.Release(instance);
        }

        [Test]
        public void Load_AllDemoDefinitions_AreMesh3D()
        {
            var loader = new CharacterDefinitionLoader();
            foreach (var pilotId in new[] { "demo_scout_01", "demo_striker_01", "demo_guard_01" })
            {
                var definition = loader.Load(pilotId);
                Assert.That(definition.AssetMode, Is.EqualTo(CharacterAssetMode.Mesh3D));
                Assert.That(definition.MeshPath, Does.EndWith(".glb"));
                Assert.That(definition.BehaviorProfileId, Is.Not.Empty);
                Assert.That(definition.MaterialProfileId, Is.Not.Empty);
            }
        }
    }
}
