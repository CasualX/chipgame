package net.casualhacks.chipdx

object ChipJNI {
	init {
		System.loadLibrary("chipjni")
	}

	external fun nativeCreate(host: Any): Long
	external fun nativeDestroy(handle: Long)
	external fun nativeAudioInit(handle: Long)
	external fun nativeOnSurfaceCreated(handle: Long): Boolean
	external fun nativeOnSurfaceChanged(handle: Long, width: Int, height: Int)
	external fun nativeSetButtons(handle: Long, buttons: Int)
	external fun nativeFrame(handle: Long, frameTimeNanos: Long)
}
