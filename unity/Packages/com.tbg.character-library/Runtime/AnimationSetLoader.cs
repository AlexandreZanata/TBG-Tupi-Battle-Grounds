using System;
using System.Collections.Generic;
using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class AnimationSetManifest
    {
        public string Id { get; init; }
        public string BodyClass { get; init; }
        public string ClipSource { get; init; }
        public IReadOnlyDictionary<string, string> Clips { get; init; }

        public string ResolveClipName(string actionKey)
        {
            if (Clips != null && Clips.TryGetValue(actionKey, out var clipName))
            {
                return clipName;
            }

            return null;
        }
    }

    [Serializable]
    public sealed class AnimationSetClipsRecord
    {
        public string idle;
        public string walk;
        public string spawn;
        public string attack;
        public string attack2;
        public string ultimate;
        public string death;
    }

    [Serializable]
    public sealed class AnimationSetJsonRecord
    {
        public string id;
        public string bodyClass;
        public string clipSource;
        public AnimationSetClipsRecord clips;
    }

    public sealed class AnimationSetLoader
    {
        public AnimationSetManifest Load(string animationSetId)
        {
            if (string.IsNullOrWhiteSpace(animationSetId))
            {
                throw new ArgumentException("Animation set id is required.", nameof(animationSetId));
            }

            var path = Path.Combine(
                CharacterPaths.GetRepoRoot(),
                "content/characters/animation_sets",
                animationSetId + ".json");
            if (!File.Exists(path))
            {
                throw new FileNotFoundException("Animation set not found.", path);
            }

            var record = JsonUtility.FromJson<AnimationSetJsonRecord>(File.ReadAllText(path));
            ValidateOrThrow(record);
            var manifest = FromRecord(record);
            BodyClassClipRegistry.Register(record.bodyClass, record.id);
            return manifest;
        }

        public static AnimationSetManifest FromRecord(AnimationSetJsonRecord record)
        {
            var clips = BuildClipMap(record.clips);
            return new AnimationSetManifest
            {
                Id = record.id,
                BodyClass = record.bodyClass,
                ClipSource = record.clipSource,
                Clips = clips
            };
        }

        public static void ValidateOrThrow(AnimationSetJsonRecord record)
        {
            var errors = Validate(record);
            if (errors.Count > 0)
            {
                throw new InvalidOperationException(string.Join("; ", errors));
            }
        }

        public static List<string> Validate(AnimationSetJsonRecord record)
        {
            var errors = new List<string>();
            if (record == null || string.IsNullOrWhiteSpace(record.id))
            {
                errors.Add("animation set id is required");
                return errors;
            }

            if (string.IsNullOrWhiteSpace(record.bodyClass))
            {
                errors.Add("bodyClass is required");
            }

            return errors;
        }

        private static Dictionary<string, string> BuildClipMap(AnimationSetClipsRecord clips)
        {
            var map = new Dictionary<string, string>();
            if (clips == null)
            {
                return map;
            }

            AddClip(map, "idle", clips.idle);
            AddClip(map, "walk", clips.walk);
            AddClip(map, "spawn", clips.spawn);
            AddClip(map, "attack", clips.attack);
            AddClip(map, "attack2", clips.attack2);
            AddClip(map, "ultimate", clips.ultimate);
            AddClip(map, "death", clips.death);
            return map;
        }

        private static void AddClip(Dictionary<string, string> map, string key, string value)
        {
            if (!string.IsNullOrWhiteSpace(value))
            {
                map[key] = value;
            }
        }
    }
}
