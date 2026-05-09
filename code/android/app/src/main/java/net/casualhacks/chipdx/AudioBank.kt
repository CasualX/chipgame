package net.casualhacks.chipdx

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioManager
import android.media.MediaPlayer
import android.media.SoundPool
import java.io.File

class AudioBank(context: Context) {
	private val audioDir = File(context.cacheDir, "audio")
	private val soundPool = SoundPool.Builder()
		.setAudioAttributes(
			AudioAttributes.Builder()
				.setLegacyStreamType(AudioManager.STREAM_MUSIC)
				.build(),
		)
		.setMaxStreams(8)
		.build()
	private val sounds = mutableMapOf<Int, Int>()
	private val musicFiles = mutableMapOf<Int, File>()
	private var activeMusicId: Int? = null
	private var mediaPlayer: MediaPlayer? = null

	init {
		audioDir.mkdirs()
	}

	fun registerSound(id: Int, relativePath: String, data: ByteArray) {
		val file = writeAudioFile("sound", id, relativePath, data)
		sounds[id]?.let(soundPool::unload)
		sounds[id] = soundPool.load(file.absolutePath, 1)
	}

	fun registerMusic(id: Int, relativePath: String, data: ByteArray) {
		musicFiles[id] = writeAudioFile("music", id, relativePath, data)
	}

	fun playSound(id: Int) {
		val soundId = sounds[id] ?: return
		soundPool.play(soundId, 1f, 1f, 1, 0, 1f)
	}

	fun playMusic(id: Int) {
		if (id < 0) {
			stopMusic()
			return
		}
		if (activeMusicId == id) {
			return
		}
		val file = musicFiles[id] ?: return
		stopMusic()
		mediaPlayer = MediaPlayer().apply {
			setDataSource(file.absolutePath)
			isLooping = true
			setVolume(0.375f, 0.375f)
			prepare()
			start()
		}
		activeMusicId = id
	}

	fun onPause() {
		mediaPlayer?.pause()
	}

	fun onResume() {
		mediaPlayer?.start()
	}

	fun release() {
		stopMusic()
		soundPool.release()
	}

	private fun stopMusic() {
		mediaPlayer?.stop()
		mediaPlayer?.release()
		mediaPlayer = null
		activeMusicId = null
	}

	private fun writeAudioFile(kind: String, id: Int, relativePath: String, data: ByteArray): File {
		val suffix = relativePath.substringAfterLast('.', "bin")
		val file = File(audioDir, "${kind}_${id}.${suffix}")
		file.writeBytes(data)
		return file
	}
}
