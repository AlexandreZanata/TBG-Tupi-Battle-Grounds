using System;

namespace TBG.CharacterLibrary
{
    [Serializable]
    public sealed class BehaviorActionsRecord
    {
        public string idle;
        public string spawn;
        public string attack;
        public string attack2;
        public string attack3;
        public string death;
        public string walk;
        public string ultimate;
    }

    [Serializable]
    public sealed class BehaviorTriggersRecord
    {
        public string idle;
        public string spawn;
        public string attack;
        public string death;
        public string walk;
        public string ultimate;
    }

    [Serializable]
    public sealed class BehaviorTimingsRecord
    {
        public int attackWindupMs;
        public int attackHitMs;
        public int ultimateWindupMs;
        public int ultimateDurationMs;
    }

    [Serializable]
    public sealed class BehaviorCombatRecord
    {
        public string attackCycle;
    }

    [Serializable]
    public sealed class BehaviorLocomotionRecord
    {
        public bool useRootMotionWalk;
    }

    [Serializable]
    public sealed class BehaviorProfileJsonRecord
    {
        public string id;
        public string archetype;
        public string bodyClass;
        public string animatorController;
        public string clipSource;
        public BehaviorActionsRecord actions;
        public BehaviorTriggersRecord triggers;
        public BehaviorTimingsRecord timings;
        public BehaviorCombatRecord combat;
        public BehaviorLocomotionRecord locomotion;
    }
}
