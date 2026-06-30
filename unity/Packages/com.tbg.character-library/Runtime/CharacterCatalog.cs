using TBG.CharacterSystem.Domain;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class CharacterCatalog
    {
        private readonly CharacterDefinitionLoader _loader;
        private readonly ProceduralCharacterBuilder _proceduralBuilder;
        private readonly Mesh3DCharacterFactory _mesh3DFactory;
        private readonly BehaviorProfileLoader _behaviorLoader;

        public CharacterCatalog()
            : this(
                new CharacterDefinitionLoader(),
                new ProceduralCharacterBuilder(),
                new Mesh3DCharacterFactory(),
                new BehaviorProfileLoader())
        {
        }

        public CharacterCatalog(
            CharacterDefinitionLoader loader,
            ProceduralCharacterBuilder proceduralBuilder,
            Mesh3DCharacterFactory mesh3DFactory,
            BehaviorProfileLoader behaviorLoader)
        {
            _loader = loader;
            _proceduralBuilder = proceduralBuilder;
            _mesh3DFactory = mesh3DFactory;
            _behaviorLoader = behaviorLoader;
        }

        public CharacterInstance Spawn(string id, Transform parent)
        {
            var definition = _loader.Load(id);
            var root = definition.AssetMode == CharacterAssetMode.Mesh3D
                ? _mesh3DFactory.Create(definition, parent)
                : SpawnProcedural(definition, parent);
            var behaviorProfile = ResolveBehaviorProfile(definition);
            var behavior = new BehaviorPlayer(root, behaviorProfile);
            return new CharacterInstance(id, root, behavior);
        }

        public void Release(CharacterInstance instance)
        {
            if (instance?.Root == null)
            {
                return;
            }

            Object.Destroy(instance.Root);
        }

        private GameObject SpawnProcedural(LoadedCharacterDefinition definition, Transform parent)
        {
            var scriptable = ScriptableObject.CreateInstance<CharacterDefinition>();
            scriptable.ApplyData(definition.ProceduralData);
            var root = _proceduralBuilder.Build(scriptable);
            root.transform.SetParent(parent, false);
            Object.Destroy(scriptable);
            return root;
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
