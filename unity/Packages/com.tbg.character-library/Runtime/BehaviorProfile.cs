using System;

namespace TBG.CharacterLibrary
{
    public sealed class BehaviorProfile
    {
        public string Id { get; init; }
        public string Archetype { get; init; }
        public string BodyClass { get; init; }
        public string AnimatorControllerPath { get; init; }
        public string ClipSource { get; init; }
        public string AttackCycle { get; init; } = "repeat";
        public int AttackWindupMs { get; init; }
        public int AttackHitMs { get; init; }
        public int UltimateWindupMs { get; init; }
        public int UltimateDurationMs { get; init; }

        private readonly string[] _stateNames;
        private readonly string[] _triggerNames;
        private readonly string _attack2State;

        public static BehaviorProfile CreateDefault()
        {
            return new BehaviorProfile
            {
                Id = "default",
                ClipSource = "embedded",
                _stateNames = new[] { "Idle", "Spawn", "Attack", "Death", "Walk", "Ultimate" },
                _triggerNames = new[] { "Idle", "Spawn", "Attack", "Death", "Walk", "Ultimate" },
                _attack2State = "Attack2"
            };
        }

        public static BehaviorProfile FromRecord(BehaviorProfileJsonRecord record)
        {
            if (record == null || string.IsNullOrWhiteSpace(record.id))
            {
                throw new ArgumentException("Behavior profile id is required.");
            }

            var actions = record.actions ?? new BehaviorActionsRecord();
            var triggers = record.triggers ?? new BehaviorTriggersRecord();
            var timings = record.timings ?? new BehaviorTimingsRecord();
            var combat = record.combat ?? new BehaviorCombatRecord();
            return new BehaviorProfile
            {
                Id = record.id,
                Archetype = record.archetype,
                BodyClass = record.bodyClass,
                AnimatorControllerPath = record.animatorController,
                ClipSource = record.clipSource,
                AttackCycle = string.IsNullOrWhiteSpace(combat.attackCycle) ? "repeat" : combat.attackCycle,
                AttackWindupMs = timings.attackWindupMs,
                AttackHitMs = timings.attackHitMs,
                UltimateWindupMs = timings.ultimateWindupMs,
                UltimateDurationMs = timings.ultimateDurationMs,
                _stateNames = new[]
                {
                    actions.idle,
                    actions.spawn,
                    actions.attack,
                    actions.death,
                    ResolveOptionalAction(actions.walk, "Walk"),
                    ResolveOptionalAction(actions.ultimate, "Ultimate")
                },
                _triggerNames = new[]
                {
                    triggers.idle,
                    triggers.spawn,
                    triggers.attack,
                    triggers.death,
                    ResolveOptionalTrigger(triggers.walk, "Walk"),
                    ResolveOptionalTrigger(triggers.ultimate, "Ultimate")
                },
                _attack2State = ResolveOptionalAction(actions.attack2, "Attack2")
            };
        }

        public string GetStateName(CharacterAction action) => ResolveName(_stateNames, action);

        public string GetTriggerName(CharacterAction action) => ResolveName(_triggerNames, action);

        public string GetAttackStateName(int index)
        {
            return index > 0 ? _attack2State : GetStateName(CharacterAction.Attack);
        }

        private static string ResolveName(string[] names, CharacterAction action)
        {
            var index = (int)action;
            if (names == null || index < 0 || index >= names.Length)
            {
                return action.ToString();
            }

            var value = names[index];
            return string.IsNullOrWhiteSpace(value) ? action.ToString() : value;
        }

        private static string ResolveOptionalAction(string value, string fallback)
        {
            return string.IsNullOrWhiteSpace(value) ? fallback : value;
        }

        private static string ResolveOptionalTrigger(string value, string fallback)
        {
            return string.IsNullOrWhiteSpace(value) ? fallback : value;
        }
    }
}
