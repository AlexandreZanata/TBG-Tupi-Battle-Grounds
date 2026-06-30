using System;
using System.Collections.Generic;
using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class BehaviorProfileLoader
    {
        private static readonly string[] RequiredActions = { "idle", "spawn", "attack", "death" };

        public BehaviorProfile Load(string profileId)
        {
            if (string.IsNullOrWhiteSpace(profileId))
            {
                throw new ArgumentException("Behavior profile id is required.", nameof(profileId));
            }

            var path = Path.Combine(CharacterPaths.BehaviorsDir, profileId + ".json");
            if (!File.Exists(path))
            {
                throw new FileNotFoundException("Behavior profile not found.", path);
            }

            var json = File.ReadAllText(path);
            var record = JsonUtility.FromJson<BehaviorProfileJsonRecord>(json);
            ValidateOrThrow(record);
            return BehaviorProfile.FromRecord(record);
        }

        public static void ValidateOrThrow(BehaviorProfileJsonRecord record)
        {
            var errors = Validate(record);
            if (errors.Count > 0)
            {
                throw new InvalidOperationException(string.Join("; ", errors));
            }
        }

        public static List<string> Validate(BehaviorProfileJsonRecord record)
        {
            var errors = new List<string>();
            if (record == null || string.IsNullOrWhiteSpace(record.id))
            {
                errors.Add("behavior profile id is required");
                return errors;
            }

            if (string.IsNullOrWhiteSpace(record.animatorController))
            {
                errors.Add("animatorController is required");
            }

            if (record.actions == null)
            {
                errors.Add("actions object is required");
                return errors;
            }

            foreach (var action in RequiredActions)
            {
                var value = GetActionValue(record.actions, action);
                if (string.IsNullOrWhiteSpace(value))
                {
                    errors.Add($"actions.{action} is required");
                }
            }

            return errors;
        }

        private static string GetActionValue(BehaviorActionsRecord actions, string key)
        {
            return key switch
            {
                "idle" => actions.idle,
                "spawn" => actions.spawn,
                "attack" => actions.attack,
                "death" => actions.death,
                _ => null
            };
        }
    }
}
