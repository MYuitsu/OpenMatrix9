# Invalidation and redraw (OM9-L3-01-06-008)

`RenderScheduler::should_redraw(camera_revision, scene_revision)` returns true on the first frame, on a
camera change, on a scene change, or after `invalidate()`; `mark_drawn()` records the state so an
unchanged viewport does not redraw.

Test: `the_scheduler_redraws_only_on_change` — first true; unchanged false; camera true; scene true;
unchanged false; explicit invalidation true.
