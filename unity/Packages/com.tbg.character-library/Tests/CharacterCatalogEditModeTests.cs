using System.IO;
using NUnit.Framework;
using TBG.CharacterLibrary;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class CharacterCatalogEditModeTests
    {
        private Transform _parent;

        [SetUp]
        public void SetUp()
        {
            _parent = new GameObject("TestParent").transform;
        }

        [TearDown]
        public void TearDown()
        {
            if (_parent != null)
            {
                Object.DestroyImmediate(_parent.gameObject);
            }
        }

        [TestCase("tank_human_01")]
        [TestCase("dps_undead_01")]
        [TestCase("support_machine_01")]
        [TestCase("ranged_plant_01")]
        public void Spawn_ProceduralPilot_CreatesGeometryUnderParent(string pilotId)
        {
            var catalog = new CharacterCatalog();
            var instance = catalog.Spawn(pilotId, _parent);

            Assert.That(instance, Is.Not.Null);
            Assert.That(instance.Id, Is.EqualTo(pilotId));
            Assert.That(instance.Root.transform.parent, Is.EqualTo(_parent));
            Assert.That(instance.Root.transform.Find("Head"), Is.Not.Null);
            Assert.That(instance.Root.transform.Find("Torso"), Is.Not.Null);

            catalog.Release(instance);
        }

        [Test]
        public void Spawn_Mesh3dPilot_WhenGlbPresent_CreatesRootUnderParent()
        {
            var catalog = new CharacterCatalog();
            var instance = catalog.Spawn("test_mesh3d_spawn", _parent);

            Assert.That(instance.Id, Is.EqualTo("test_mesh3d_spawn"));
            Assert.That(instance.Root.transform.parent, Is.EqualTo(_parent));
            Assert.That(instance.Root.name, Is.EqualTo("Test Mesh3D Fixture"));
            Assert.That(instance.Root.GetComponentsInChildren<MeshFilter>().Length, Is.GreaterThan(0));

            var meshPath = Path.Combine(
                CharacterPaths.GetRepoRoot(),
                "content/characters/meshes/_test_fixture.glb");
            Assert.That(File.Exists(meshPath), Is.True);

            catalog.Release(instance);
        }

        [Test]
        public void Spawn_Mesh3dWithoutGlbFile_ThrowsFileNotFound()
        {
            var catalog = new CharacterCatalog();
            Assert.Throws<FileNotFoundException>(() => catalog.Spawn("test_mesh3d_missing_glb", _parent));
        }
    }
}
