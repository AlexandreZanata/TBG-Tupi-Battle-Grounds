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

        [Test]
        public void Spawn_ProceduralFixture_CreatesGeometryUnderParent()
        {
            var catalog = new CharacterCatalog();
            var instance = catalog.Spawn("test_procedural_spawn", _parent);

            Assert.That(instance.Id, Is.EqualTo("test_procedural_spawn"));
            Assert.That(instance.Root.transform.parent, Is.EqualTo(_parent));
            Assert.That(instance.Root.transform.Find("Head"), Is.Not.Null);
            Assert.That(instance.Root.transform.Find("Torso"), Is.Not.Null);

            catalog.Release(instance);
        }

        [Test]
        public void Spawn_Mesh3dDemo_WhenGlbPresent_CreatesRootUnderParent()
        {
            var catalog = new CharacterCatalog();
            var instance = catalog.Spawn("demo_striker_01", _parent);

            Assert.That(instance.Id, Is.EqualTo("demo_striker_01"));
            Assert.That(instance.Root.transform.parent, Is.EqualTo(_parent));
            Assert.That(instance.Root.name, Is.EqualTo("Striker Pilot"));
            Assert.That(instance.Root.GetComponentsInChildren<MeshFilter>().Length, Is.GreaterThan(0));

            var meshPath = Path.Combine(
                CharacterPaths.GetRepoRoot(),
                "content/characters/meshes/demo_striker_01.glb");
            Assert.That(File.Exists(meshPath), Is.True);

            catalog.Release(instance);
        }

        [Test]
        public void Spawn_UnknownUnit_ThrowsFileNotFound()
        {
            var catalog = new CharacterCatalog();
            Assert.Throws<FileNotFoundException>(() => catalog.Spawn("missing_unit_xyz", _parent));
        }
    }
}
