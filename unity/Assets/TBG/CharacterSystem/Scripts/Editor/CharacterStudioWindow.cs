using System;
using TBG.CharacterSystem.Domain;
using TBG.CharacterSystem.Runtime;
using UnityEditor;
using UnityEngine;

namespace TBG.CharacterSystem.Editor
{
    public class CharacterStudioWindow : EditorWindow
    {
        private string _selectedId;
        private CharacterDefinitionData _draft;
        private GameObject _previewRoot;
        private Vector2 _scroll;

        [MenuItem("TBG/Character Studio")]
        public static void Open()
        {
            GetWindow<CharacterStudioWindow>("Character Studio");
        }

        private void OnGUI()
        {
            _scroll = EditorGUILayout.BeginScrollView(_scroll);
            DrawList();
            EditorGUILayout.Space();
            DrawActions();
            if (_draft != null)
            {
                DrawEditor();
            }
            EditorGUILayout.EndScrollView();
        }

        private void DrawList()
        {
            EditorGUILayout.LabelField("Definitions", EditorStyles.boldLabel);
            foreach (var id in CharacterDefinitionRepository.ListIds())
            {
                if (GUILayout.Button(id))
                {
                    LoadDefinition(id);
                }
            }
        }

        private void DrawActions()
        {
            if (GUILayout.Button("Create Tank / Humans"))
            {
                CreateNew("tank_human_new", Archetype.Tank, Faction.Humans);
            }
            if (_draft != null && GUILayout.Button("Save"))
            {
                CharacterDefinitionRepository.Save(_draft);
                RebuildPreview();
            }
            if (_draft != null && GUILayout.Button("Delete"))
            {
                if (EditorUtility.DisplayDialog("Delete", $"Delete {_draft.id}?", "Delete", "Cancel"))
                {
                    CharacterDefinitionRepository.Delete(_draft.id);
                    ClearPreview();
                    _draft = null;
                    _selectedId = null;
                }
            }
            if (_draft != null && GUILayout.Button("Export Card (Unity)"))
            {
                ExportCard();
            }
        }

        private void DrawEditor()
        {
            EditorGUILayout.LabelField("Edit", EditorStyles.boldLabel);
            _draft.id = EditorGUILayout.TextField("Id", _draft.id);
            _draft.characterName = EditorGUILayout.TextField("Name", _draft.characterName);
            _draft.archetype = (Archetype)EditorGUILayout.EnumPopup("Archetype", _draft.archetype);
            _draft.faction = (Faction)EditorGUILayout.EnumPopup("Faction", _draft.faction);
            _draft.heightScale = EditorGUILayout.Slider("Height", _draft.heightScale, 0.1f, 3f);
            _draft.bodyWidthScale = EditorGUILayout.Slider("Body Width", _draft.bodyWidthScale, 0.1f, 3f);
            _draft.headScale = EditorGUILayout.Slider("Head", _draft.headScale, 0.1f, 3f);
            _draft.limbLengthScale = EditorGUILayout.Slider("Limbs", _draft.limbLengthScale, 0.1f, 3f);
            _draft.evolutionStage = EditorGUILayout.IntSlider("Evolution", _draft.evolutionStage, 0, 2);
        }

        private void LoadDefinition(string id)
        {
            _selectedId = id;
            _draft = CharacterDefinitionRepository.Load(id);
            RebuildPreview();
        }

        private void CreateNew(string id, Archetype archetype, Faction faction)
        {
            _draft = new CharacterDefinitionData
            {
                id = id,
                characterName = id,
                archetype = archetype,
                faction = faction,
                accessories = Array.Empty<AccessoryType>()
            };
            ArchetypeProportionDefaults.Apply(archetype, _draft);
            var colors = FactionPaletteDefaults.GetColors(faction);
            _draft.primaryColor = colors.primary;
            _draft.secondaryColor = colors.secondary;
            CharacterDefinitionRepository.Save(_draft);
            RebuildPreview();
        }

        private void RebuildPreview()
        {
            ClearPreview();
            if (_draft == null)
            {
                return;
            }

            var definition = ScriptableObject.CreateInstance<CharacterDefinition>();
            definition.ApplyData(_draft);
            var builder = new ProceduralCharacterBuilder();
            _previewRoot = builder.Build(definition);
            _previewRoot.transform.position = Vector3.zero;
        }

        private void ClearPreview()
        {
            if (_previewRoot != null)
            {
                DestroyImmediate(_previewRoot);
            }
        }

        private void ExportCard()
        {
            var renderer = FindObjectOfType<CardRenderer>();
            if (renderer == null || _previewRoot == null)
            {
                EditorUtility.DisplayDialog("Card", "Add CardRenderer + camera to scene.", "OK");
                return;
            }

            var path = System.IO.Path.Combine(CharacterPaths.CardsDir, _draft.id + ".png");
            renderer.RenderToFile(_previewRoot, path);
            AssetDatabase.Refresh();
        }

        private void OnDestroy()
        {
            ClearPreview();
        }
    }
}
