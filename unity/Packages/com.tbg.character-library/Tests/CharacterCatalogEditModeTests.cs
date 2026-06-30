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
        public void Spawn_ProceduralPilot_CreatesGeometryUnderParent()
        {
            var catalog = new CharacterCatalog();
            var instance = catalog.Spawn("tank_human_01", _parent);

            Assert.That(instance, Is.Not.Null);
            Assert.That(instance.Id, Is.EqualTo("tank_human_01"));
            Assert.That(instance.Root.transform.parent, Is.EqualTo(_parent));
            Assert.That(instance.Root.transform.Find("Head"), Is.Not.Null);
            Assert.That(instance.Root.transform.Find("Torso"), Is.Not.Null);

            catalog.Release(instance);
        }

        [Test]
        public void Load_ProceduralPilot_DefaultsToProcedural()
        {
            var loader = new CharacterDefinitionLoader();
            var definition = loader.Load("tank_human_01");

            Assert.That(definition.AssetMode, Is.EqualTo(CharacterAssetMode.Procedural));
        }

        [Test]
        public void Load_Mesh3dSchemaExample_ParsesAssetMode()
        {
            var path = Path.Combine(CharacterPaths.DefinitionsDir, "_schema.mesh3d.example.json");
            var json = File.ReadAllText(path);
            var record = JsonUtility.FromJson<CharacterDefinitionJsonRecord>(json);
            var definition = LoadedCharacterDefinition.FromRecord(record);

            Assert.That(definition.AssetMode, Is.EqualTo(CharacterAssetMode.Mesh3D));
            Assert.That(definition.MeshPath, Does.EndWith(".glb"));
        }

        [Test]
        public void Spawn_Mesh3dWithoutGlbFile_ThrowsFileNotFound()
        {
            var catalog = new CharacterCatalog();
            Assert.Throws<FileNotFoundException>(() => catalog.Spawn("_schema.mesh3d.example", _parent));
        }
    }
}
