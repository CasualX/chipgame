package net.casualhacks.chipdx

import android.content.Context
import android.opengl.GLSurfaceView
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class GameSurfaceView(
	context: Context,
	private val nativeHandle: Long,
	private val inputProvider: () -> Int,
) : GLSurfaceView(context) {
	init {
		setEGLContextClientVersion(3)
		preserveEGLContextOnPause = false
		setRenderer(GameRenderer())
		renderMode = RENDERMODE_CONTINUOUSLY
	}

	private inner class GameRenderer : Renderer {
		override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
			check(ChipJNI.nativeOnSurfaceCreated(nativeHandle)) { "Failed to create native GL surface" }
		}

		override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
			ChipJNI.nativeOnSurfaceChanged(nativeHandle, width, height)
		}

		override fun onDrawFrame(gl: GL10?) {
			ChipJNI.nativeSetButtons(nativeHandle, inputProvider())
			ChipJNI.nativeFrame(nativeHandle, System.nanoTime())
		}
	}
}
