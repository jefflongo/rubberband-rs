use std::collections::HashMap;
use std::ptr::NonNull;

use rubberband_sys as ffi;

/// Debug level for messages produced to `cerr`.
#[derive(Debug)]
#[repr(i32)]
pub enum DebugLevel {
    /// Report nothing.
    None = 0,
    /// Report errors only.
    Error = 1,
    /// Report some information on construction and ratio change. Nothing is reported during normal processing unless something changes.
    Info = 2,
    /// Report a significant amount of information about ongoing stretch calculations during normal processing.
    Verbose = 3,
    /// Report a large amount of information and also (in the R2 engine) add audible ticks to the output at phase reset points. This is seldom useful.
    Trace = 4,
}

bitflags::bitflags! {
    /// Processing options for the timestretcher.
    ///
    /// The preferred options should normally be set in [`Stretcher::new`], as a bitwise OR of the
    /// option flags. The default value ([`Self::default`]) is intended to give good results in most
    /// situations.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Options: i32 {
        /// Run the stretcher in offline mode. In this mode the input data needs to be provided
        /// twice, once to [`Stretcher::study`], which calculates a stretch profile for the audio,
        /// and once to [`Stretcher::process`], which stretches it.
        const PROCESS_OFFLINE = ffi::RubberBandOption_RubberBandOptionProcessOffline;

        /// Run the stretcher in real-time mode. In this mode only [`Stretcher::process`] should be
        /// called, and the stretcher adjusts dynamically in response to the input audio.
        const PROCESS_REALTIME = ffi::RubberBandOption_RubberBandOptionProcessRealTime;

        /// Reset component phases at the peak of each transient (the start of a significant note or
        /// percussive event). This, the default setting, usually results in a clear-sounding
        /// output; but it is not always consistent, and may cause interruptions in stable sounds
        /// present at the same time as transient events. The DETECTOR flags (below) can be used to
        /// tune this to some extent.
        const TRANSIENTS_CRISP = ffi::RubberBandOption_RubberBandOptionTransientsCrisp;

        /// Reset component phases at the peak of each transient, outside a frequency range typical
        /// of musical fundamental frequencies. The results may be more regular for mixed stable and
        /// percussive notes than [`Self::TRANSIENTS_CRISP`], but with a "phasier" sound. The
        /// balance may sound very good for certain types of music and fairly bad for others.
        const TRANSIENTS_MIXED = ffi::RubberBandOption_RubberBandOptionTransientsMixed;

        /// Do not reset component phases at any point. The results will be smoother and more
        /// regular but may be less clear than with either of the other transients flags.
        const TRANSIENTS_SMOOTH = ffi::RubberBandOption_RubberBandOptionTransientsSmooth;

        /// Use a general-purpose transient detector which is likely to be good for most situations.
        /// This is the default.
        const DETECTOR_COMPOUND = ffi::RubberBandOption_RubberBandOptionDetectorCompound;

        /// Detect percussive transients. Note that this was the default and only option in Rubber
        /// Band versions prior to 1.5.
        const DETECTOR_PERCUSSIVE = ffi::RubberBandOption_RubberBandOptionDetectorPercussive;

        /// Use an onset detector with less of a bias toward percussive transients. This may give
        /// better results with certain material (e.g. relatively monophonic piano music).
        const DETECTOR_SOFT = ffi::RubberBandOption_RubberBandOptionDetectorSoft;

        /// Adjust phases when stretching in such a way as to try to retain the continuity of phase
        /// relationships between adjacent frequency bins whose phases are behaving in similar ways.
        /// This, the default setting, should give good results in most situations.
        const PHASE_LAMINAR = ffi::RubberBandOption_RubberBandOptionPhaseLaminar;

        /// Adjust the phase in each frequency bin independently from its neighbours. This usually
        /// results in a slightly softer, phasier sound.
        const PHASE_INDEPENDENT = ffi::RubberBandOption_RubberBandOptionPhaseIndependent;

        /// Permit the stretcher to determine its own threading model. In the R2 engine this means
        /// using one processing thread per audio channel in offline mode if the stretcher is able
        /// to determine that more than one CPU is available, and one thread only in realtime mode.
        /// The R3 engine does not currently have a multi-threaded mode, but if one is introduced in
        /// future, this option may use it. This is the default.
        const THREADING_AUTO = ffi::RubberBandOption_RubberBandOptionThreadingAuto;

        /// Never use more than one thread.
        const THREADING_NEVER = ffi::RubberBandOption_RubberBandOptionThreadingNever;

        /// Use multiple threads in any situation where OptionThreadingAuto would do so, except omit
        /// the check for multiple CPUs and instead assume it to be true.
        const THREADING_ALWAYS = ffi::RubberBandOption_RubberBandOptionThreadingAlways;

        /// Use the default window size. The actual size will vary depending on other parameters.
        /// This option is expected to produce better results than the other window options in most
        /// situations. In the R3 engine this causes the engine's full multi-resolution processing
        /// scheme to be used.
        const WINDOW_STANDARD = ffi::RubberBandOption_RubberBandOptionWindowStandard;

        /// Use a shorter window. This has different effects with R2 and R3 engines.
        const WINDOW_SHORT = ffi::RubberBandOption_RubberBandOptionWindowShort;

        /// Use a longer window. With the R2 engine this is likely to result in a smoother sound at
        /// the expense of clarity and timing. The R3 engine currently ignores this option, treating
        /// it like [`Self::WINDOW_STANDARD`].
        const WINDOW_LONG = ffi::RubberBandOption_RubberBandOptionWindowLong;

        /// Do not use time-domain smoothing. This is the default.
        const SMOOTHING_OFF = ffi::RubberBandOption_RubberBandOptionSmoothingOff;

        /// Use time-domain smoothing. This will result in a softer sound with some audible
        /// artifacts around sharp transients, but it may be appropriate for longer stretches of
        /// some instruments and can mix well with [`Self::WINDOW_SHORT`].
        const SMOOTHING_ON = ffi::RubberBandOption_RubberBandOptionSmoothingOn;

        /// Apply no special formant processing. The spectral envelope will be pitch shifted as
        /// normal. This is the default.
        const FORMANT_SHIFTED = ffi::RubberBandOption_RubberBandOptionFormantShifted;

        /// Preserve the spectral envelope of the unshifted signal. This permits shifting the note
        /// frequency without so substantially affecting the perceived pitch profile of the voice or
        /// instrument.
        const FORMANT_PRESERVED = ffi::RubberBandOption_RubberBandOptionFormantPreserved;

        /// Favour CPU cost over sound quality. This is the default. Use this when time-stretching
        /// only, or for fixed pitch shifts where CPU usage is of concern. Do not use this for
        /// arbitrarily time-varying pitch shifts (see [`Self::PITCH_HIGH_CONSISTENCY`] below).
        const PITCH_HIGH_SPEED = ffi::RubberBandOption_RubberBandOptionPitchHighSpeed;

        /// Favour sound quality over CPU cost. Use this for fixed pitch shifts where sound quality
        /// is of most concern. Do not use this for arbitrarily time-varying pitch shifts (see
        /// [`Self::PITCH_HIGH_CONSISTENCY`] below).
        const PITCH_HIGH_QUALITY = ffi::RubberBandOption_RubberBandOptionPitchHighQuality;

        /// Use a method that supports dynamic pitch changes without discontinuities, including when
        /// crossing the 1.0 pitch scale. This may cost more in CPU than the default, especially
        /// when the pitch scale is exactly 1.0. You should use this option whenever you wish to
        /// support dynamically changing pitch shift during processing.
        const PITCH_HIGH_CONSISTENCY = ffi::RubberBandOption_RubberBandOptionPitchHighConsistency;

        /// Channels are handled for maximum individual fidelity, at the expense of synchronisation.
        /// In the R3 engine, this means frequency-bin synchronisation is maintained more closely
        /// for lower-frequency content than higher. In R2, it means the stereo channels are
        /// processed individually and only synchronised at transients. In both engines this gives
        /// the highest quality for the individual channels but a more diffuse stereo image, an
        /// unnatural increase in "width", and generally a loss of mono compatibility (i.e. mono
        /// mixes from stereo can sound phasy). This option is the default.
        const CHANNELS_APART = ffi::RubberBandOption_RubberBandOptionChannelsApart;

        /// Channels are handled for higher synchronisation at some expense of individual fidelity.
        /// In particular, a stretcher processing two channels will treat its input as a stereo pair
        /// and aim to maximise clarity at the centre and preserve mono compatibility. This gives
        /// relatively less stereo space and width than the default, as well as slightly lower
        /// fidelity for individual channel content, but the results may be more appropriate for
        /// many situations making use of stereo mixes.
        const CHANNELS_TOGETHER = ffi::RubberBandOption_RubberBandOptionChannelsTogether;

        /// Use the Rubber Band Library R2 (Faster) engine. This is the engine implemented in Rubber
        /// Band Library v1.x and v2.x, and it remains the default in newer versions. It uses
        /// substantially less CPU than the R3 engine and there are still many situations in which
        /// it is likely to be the more appropriate choice.
        const ENGINE_FASTER = ffi::RubberBandOption_RubberBandOptionEngineFaster;

        /// Use the Rubber Band Library R3 (Finer) engine. This engine was introduced in Rubber Band
        /// Library v3.0. It produces higher-quality results than the R2 engine for most material,
        /// especially complex mixes, vocals and other sounds that have soft onsets and smooth pitch
        /// changes, and music with substantial bass content. However, it uses much more CPU power
        /// than the R2 engine.
        const ENGINE_FINER = ffi::RubberBandOption_RubberBandOptionEngineFiner;
    }
}

/// Construct a time and pitch stretcher object to run at the given sample rate, with the given
/// number of channels.
///
/// Both of the stretcher engines provide their best balance of quality with efficiency at sample
/// rates of 44100 or 48000 Hz. Other rates may be used, and the stretcher should produce sensible
/// output with any rate from 8000 to 192000 Hz, but you are advised to use 44100 or 48000 where
/// practical. Do not use rates below 8000 or above 192000 Hz.
///
/// Initial time and pitch scaling ratios and other processing options may be provided.
/// In particular, the behaviour of the stretcher depends strongly on whether offline or real-time
/// mode is selected on construction (via [`Options::PROCESS_OFFLINE`] or
/// [`Options::PROCESS_REALTIME`] option - offline is the default).
///
/// In offline mode, you must provide the audio block-by-block in two passes: in the first pass
/// calling [`Self::study`], in the second pass calling [`Self::process`] and receiving the output
/// via [`Self::retrieve`]. In real-time mode, there is no study pass, just a single streaming pass
/// in which the audio is passed to [`Self::process`] and output received via [`Self::retrieve`].
///
/// In real-time mode you can change the time and pitch ratios at any time, but in offline mode they
/// are fixed and cannot be changed after the study pass has begun. (However, see
/// [`Self::setKeyFrameMap`] for a way to do pre-planned variable time stretching in offline mode.)
pub struct Stretcher {
    state: NonNull<ffi::RubberBandState_>,
}

impl Stretcher {
    /// Constructs a new [`Stretcher`].
    pub fn new(
        sample_rate: u32,
        channels: u32,
        options: Options,
        initial_time_ratio: f64,
        initial_pitch_scale: f64,
    ) -> Self {
        let raw = unsafe {
            ffi::rubberband_new(
                sample_rate,
                channels,
                options.bits(),
                initial_time_ratio,
                initial_pitch_scale,
            )
        };
        Stretcher {
            state: NonNull::new(raw).expect("rubberband_new returned null pointer"),
        }
    }

    #[inline]
    fn as_ptr(&self) -> ffi::RubberBandState {
        self.state.as_ptr()
    }

    /// Reset the stretcher's internal buffers.
    ///
    /// The stretcher should subsequently behave as if it had just been constructed (although
    /// retaining the current time and pitch ratio).
    pub fn reset(&mut self) {
        unsafe { ffi::rubberband_reset(self.as_ptr()) }
    }

    /// Return the active internal engine version, according to the `ENGINE` flag supplied on
    /// construction.
    ///
    /// This will return 2 for the R2 (Faster) engine or 3 for the R3 (Finer) engine.
    pub fn engine_version(&self) -> i32 {
        unsafe { ffi::rubberband_get_engine_version(self.as_ptr()) }
    }

    /// Set the time ratio for the stretcher.
    ///
    /// This is the ratio of stretched to unstretched duration – not tempo. For example, a ratio of
    /// 2.0 would make the audio twice as long (i.e. halve the tempo); 0.5 would make it half as
    /// long (i.e. double the tempo); 1.0 would leave the duration unaffected.
    ///
    /// If the stretcher was constructed in Offline mode, the time ratio is fixed throughout
    /// operation; this function may be called any number of times between construction (or a call
    /// to [`Self::reset`]) and the first call to [`Self::study`] or [`Self::process`], but may not
    /// be called after [`Self::study`] or [`Self::process`] has been called.
    ///
    /// If the stretcher was constructed in RealTime mode, the time ratio may be varied during
    /// operation; this function may be called at any time, so long as it is not called concurrently
    /// with [`Self::process`]. You should either call this function from the same thread as
    /// [`Self::process`], or provide your own mutex or similar mechanism to ensure that
    /// [`Self::set_time_ratio`] and [`Self::process`] cannot be run at once (there is no internal
    /// mutex for this purpose).
    pub fn set_time_ratio(&mut self, ratio: f64) {
        unsafe { ffi::rubberband_set_time_ratio(self.as_ptr(), ratio) }
    }

    /// Set the pitch scaling ratio for the stretcher.
    ///
    /// This is the ratio of target frequency to source frequency. For example, a ratio of 2.0 would
    /// shift up by one octave; 0.5 down by one octave; or 1.0 leave the pitch unaffected.
    ///
    /// To put this in musical terms, a pitch scaling ratio corresponding to a shift of S
    /// equal-tempered semitones (where S is positive for an upwards shift and negative for
    /// downwards) is `pow(2.0, S / 12.0)``.
    ///
    /// If the stretcher was constructed in Offline mode, the pitch scaling ratio is fixed
    /// throughout operation; this function may be called any number of times between construction
    /// (or a call to [`Self::reset`]) and the first call to [`Self::study`] or [`Self::process`],
    /// but may not be called after [`Self::study`] or [`Self::process`]) has been called.
    ///
    /// If the stretcher was constructed in RealTime mode, the pitch scaling ratio may be varied
    /// during operation; this function may be called at any time, so long as it is not called
    /// concurrently with [`Self::process`]. You should either call this function from the same
    /// thread as [`Self::process`], or provide your own mutex or similar mechanism to ensure that
    /// [`Self::set_pitch_scale`] and [`Self::process`] cannot be run at once (there is no internal
    /// mutex for this purpose).
    pub fn set_pitch_scale(&mut self, scale: f64) {
        unsafe { ffi::rubberband_set_pitch_scale(self.as_ptr(), scale) }
    }

    /// Set a pitch scale for the vocal formant envelope separately from the overall pitch scale.
    ///
    /// This is a ratio of target frequency to source frequency. For example, a ratio of 2.0 would
    /// shift the formant envelope up by one octave; 0.5 down by one octave; or 1.0 leave the
    /// formant unaffected.
    ///
    /// By default this is set to the special value of 0.0, which causes the scale to be calculated
    /// automatically. It will be treated as 1.0 / the pitch scale if [`Options::FORMANT_PRESERVED`]
    /// is specified, or 1.0 for [`Options::FORMANT_SHIFTED`].
    ///
    /// Conversely, if this is set to a value other than the default 0.0, formant shifting will
    /// happen regardless of the state of the [`Options::FORMANT_PRESERVED`] /
    /// [`Options::FORMANT_SHIFTED`] option.
    ///
    /// This function is provided for special effects only. You do not need to call it for ordinary
    /// pitch shifting, with or without formant preservation - just specify or omit the
    /// [`Options::FORMANT_PRESERVED`] option as appropriate. Use this function only if you want to
    /// shift formants by a distance other than that of the overall pitch shift.
    ///
    /// This function is supported only in the R3 ([`Options::ENGINE_FINER`]) engine. It has no
    /// effect in R2 ([`Options::ENGINE_FASTER`]).
    pub fn set_formant_scale(&mut self, scale: f64) {
        unsafe { ffi::rubberband_set_formant_scale(self.as_ptr(), scale) }
    }

    /// Return the last time ratio value that was set (either on construction or with
    /// [`Self::set_time_ratio`]).
    pub fn time_ratio(&self) -> f64 {
        unsafe { ffi::rubberband_get_time_ratio(self.as_ptr()) }
    }

    /// Return the last pitch scaling ratio value that was set (either on construction or with
    /// [`Self::set_pitch_scale`]).
    pub fn pitch_scale(&self) -> f64 {
        unsafe { ffi::rubberband_get_pitch_scale(self.as_ptr()) }
    }

    /// Return the last formant scaling ratio that was set with setFormantScale, or 0.0 if the
    /// default automatic scaling is in effect.
    ///
    /// This function is supported only in the R3 ([`Options::ENGINE_FINER`]) engine. It always
    /// returns 0.0 in R2 ([`Options::ENGINE_FASTER`]).
    pub fn formant_scale(&self) -> f64 {
        unsafe { ffi::rubberband_get_formant_scale(self.as_ptr()) }
    }

    /// In RealTime mode (unlike in Offline mode) the stretcher performs no automatic padding or
    /// delay/latency compensation at the start of the signal.
    ///
    /// This permits applications to have their own custom requirements, but it also means that by
    /// default some samples will be lost or attenuated at the start of the output and the correct
    /// linear relationship between input and output sample counts may be lost.
    ///
    /// Most applications using RealTime mode should solve this by calling
    /// [`Self::get_preferred_start_pad`] and supplying the returned number of (silent) samples at
    /// the start of their input, before their first "true" [`Self::process`] call; and then also
    /// calling [`Self::get_start_delay`] and trimming the returned number of samples from the start
    /// of their stretcher's output.
    ///
    /// Ensure you have set the time and pitch scale factors to their proper starting values before
    /// calling [`Self::get_preferred_start_pad`] or [`Self::get_start_delay`].
    ///
    /// In Offline mode, padding and delay compensation are handled internally and both functions
    /// always return zero.
    pub fn get_preferred_start_pad(&self) -> u32 {
        unsafe { ffi::rubberband_get_preferred_start_pad(self.as_ptr()) }
    }

    /// Return the output delay of the stretcher.
    ///
    /// This is the number of audio samples that one should discard at the start of the output,
    /// after padding the start of the input with [`Self::get_preferred_start_pad`], in order to
    /// ensure that the resulting audio has the expected time alignment with the input.
    ///
    /// Ensure you have set the time and pitch scale factors to their proper starting values before
    /// calling [`Self::get_preferred_start_pad`] or [`Self::get_start_delay`].
    ///
    /// In Offline mode, padding and delay compensation are handled internally and both functions
    /// always return zero.
    pub fn get_start_delay(&self) -> u32 {
        unsafe { ffi::rubberband_get_start_delay(self.as_ptr()) }
    }

    /// Return the number of channels this stretcher was constructed with.
    pub fn get_channel_count(&self) -> u32 {
        unsafe { ffi::rubberband_get_channel_count(self.as_ptr()) }
    }

    /// Change a `TRANSIENTS` configuration setting.
    ///
    /// This may be called at any time in RealTime mode. It may not be called in Offline mode
    /// (for which the transients option is fixed on construction). This has no effect when using
    /// the R3 engine.
    pub fn set_transients_option(&self, options: Options) {
        unsafe {
            ffi::rubberband_set_transients_option(self.as_ptr(), options.bits());
        }
    }

    /// Change a `DETECTOR` configuration setting.
    ///
    /// This may be called at any time in RealTime mode. It may not be called in Offline mode (for
    /// which the detector option is fixed on construction). This has no effect when using the R3
    /// engine.
    pub fn set_detector_option(&self, options: Options) {
        unsafe {
            ffi::rubberband_set_detector_option(self.as_ptr(), options.bits());
        }
    }

    /// Change a `PHASE` configuration setting.
    ///
    /// This may be called at any time in any mode. This has no effect when using the R3 engine.
    ///
    /// Note that if running multi-threaded in Offline mode, the change may not take effect
    /// immediately if processing is already under way when this function is called.
    pub fn set_phase_option(&self, options: Options) {
        unsafe {
            ffi::rubberband_set_phase_option(self.as_ptr(), options.bits());
        }
    }

    /// Change a `FORMANT` configuration setting.
    ///
    /// This may be called at any time in any mode.
    ///
    /// Note that if running multi-threaded in Offline mode, the change may not take effect
    /// immediately if processing is already under way when this function is called.
    pub fn set_formant_option(&self, options: Options) {
        unsafe {
            ffi::rubberband_set_formant_option(self.as_ptr(), options.bits());
        }
    }

    /// Change a PITCH configuration setting.
    ///
    /// This may be called at any time in RealTime mode. It may not be called in Offline mode (for
    /// which the pitch option is fixed on construction). This has no effect when using the R3
    /// engine.
    pub fn set_pitch_option(&self, options: Options) {
        unsafe {
            ffi::rubberband_set_pitch_option(self.as_ptr(), options.bits());
        }
    }

    /// Tell the stretcher exactly how many input sample frames it will receive.
    ///
    /// This is only useful in Offline mode, when it allows the stretcher to ensure that the number
    /// of output samples is exactly correct. In RealTime mode no such guarantee is possible and
    /// this value is ignored.
    ///
    /// Note that the value of `samples` refers to the number of audio sample frames, which may be
    /// multi-channel, not the number of individual samples. (For example, one second of stereo
    /// audio sampled at 44100Hz yields a value of 44100 sample frames, not 88200.) This rule
    /// applies throughout the Rubber Band API.
    pub fn set_expected_input_duraction(&self, samples: u32) {
        unsafe {
            ffi::rubberband_set_expected_input_duration(self.as_ptr(), samples);
        }
    }

    /// Tell the stretcher the maximum number of sample frames that you will ever be passing in to a
    /// single [`Self::process`] call.
    ///
    /// If you don't call this, the stretcher will assume that you are calling
    /// [`Self::get_samples_required`] at each cycle and are never passing more samples than are
    /// suggested by that function.
    ///
    /// If your application has some external constraint that means you prefer a fixed block size,
    /// then your normal mode of operation would be to provide that block size to this function; to
    /// loop calling [`Self::process`] with that size of block; after each call to
    /// [`Self::process`], test whether output has been generated by calling [`Self::available`];
    /// and, if so, call [`Self::retrieve`] to obtain it. See [`Self::get_samples_required`] for a
    /// more suitable operating mode for applications without such external constraints.
    ///
    /// This function may not be called after the first call to [`Self::study`] or [`Self::process`].
    ///
    /// Note that this value is only relevant to [`Self::process`], not to [`Self::study`] (to which
    /// you may pass any number of samples at a time, and from which there is no output).
    ///
    /// Despite the existence of this call and its use of a `u32` argument, there is an internal
    /// limit to the maximum process buffer size that can be requested. Call
    /// [`Self::get_process_size_limit`] to query that limit. The Rubber Band API is essentially
    /// block-based and is not designed to process an entire signal within a single process cycle.
    ///
    /// Note that the value of `samples` refers to the number of audio sample frames, which may be
    /// multi-channel, not the number of individual samples. (For example, one second of stereo
    /// audio sampled at 44100Hz yields a value of 44100 sample frames, not 88200.) This rule
    /// applies throughout the Rubber Band API.
    pub fn set_max_process_size(&self, samples: u32) {
        unsafe { ffi::rubberband_set_max_process_size(self.as_ptr(), samples) }
    }

    /// Obtain the overall maximum supported process buffer size in sample frames, which is also the
    /// maximum acceptable value to pass to [`Self::set_max_process_size`].
    ///
    /// This value is fixed across instances and configurations. As of Rubber Band v3.3 it is always
    /// 524288 (or 2^19), but in principle it may change in future releases.
    pub fn get_process_size_limit(&self) -> u32 {
        unsafe { ffi::rubberband_get_process_size_limit(self.as_ptr()) }
    }

    /// Ask the stretcher how many audio sample frames should be provided as input in order to
    /// ensure that some more output becomes available.
    ///
    /// If your application has no particular constraint on processing block size and you are able
    /// to provide any block size as input for each cycle, then your normal mode of operation would
    /// be to loop querying this function; providing that number of samples to [`Self::process`];
    /// and reading the output (repeatedly if necessary) using [`Self::available`] and
    /// [`Self::retrieve`]. See [`Self::set_max_process_size`] for a more suitable operating mode
    /// for applications that do have external block size constraints.
    ///
    /// Note that this value is only relevant to [`Self::process`], not to [`Self::study`] (to which
    /// you may pass any number of samples at a time, and from which there is no output).
    ///
    /// Note that the return value refers to the number of audio sample frames, which may be
    /// multi-channel, not the number of individual samples. (For example, one second of stereo
    /// audio sampled at 44100Hz yields a value of 44100 sample frames, not 88200.) This rule
    /// applies throughout the Rubber Band API.
    pub fn get_samples_required(&self) -> u32 {
        unsafe { ffi::rubberband_get_samples_required(self.as_ptr()) }
    }

    /// Provide a set of mappings from "before" to "after" sample numbers so as to enforce a
    /// particular stretch profile.
    ///
    /// The argument is a map from audio sample frame number in the source material, to the
    /// corresponding sample frame number in the stretched output. The mapping should be for key
    /// frames only, with a "reasonable" gap between mapped samples.
    ///
    /// This function cannot be used in RealTime mode.
    ///
    /// This function may not be called after the first call to [`Self::process`]. It should be
    /// called after the time and pitch ratios have been set; the results of changing the time and
    /// pitch ratios after calling this function are undefined. Calling [`Self::reset`] will clear
    /// this mapping.
    ///
    /// The key frame map only affects points within the material; it does not determine the overall
    /// stretch ratio (that is, the ratio between the output material's duration and the source
    /// material's duration). You need to provide this ratio separately to [`Self::set_time_ratio`],
    /// otherwise the results may be truncated or extended in unexpected ways regardless of the
    /// extent of the frame numbers found in the key frame map.
    pub fn set_key_frame_map(&self, map: HashMap<u32, u32>) {
        unsafe {
            ffi::rubberband_set_key_frame_map(
                self.as_ptr(),
                map.len().try_into().unwrap(),
                map.keys().copied().collect::<Vec<_>>().as_mut_ptr(),
                map.values().copied().collect::<Vec<_>>().as_mut_ptr(),
            );
        }
    }

    /// Provide a block of sample frames for the stretcher to study and calculate a stretch profile
    /// from.
    ///
    /// This is only meaningful in Offline mode, and is required if running in that mode. You should
    /// pass the entire input through [`Self::study`] before any [`Self::process`] calls are made,
    /// as a sequence of blocks in individual [`Self::study`] calls, or as a single large block.
    ///
    /// `input` should point to de-interleaved audio data with one float array per channel. Sample
    /// values are conventionally expected to be in the range -1.0f to +1.0f.
    ///
    /// Set `is_final` to true if this is the last block of data that will be provided to
    /// [`Self::study`] before the first [`Self::process`] call.
    pub fn study(&self, input: &[&[f32]], is_final: bool) {
        let num_samples = input.iter().map(|s| s.len()).min().unwrap_or(0);
        let input = input.iter().map(|slice| slice.as_ptr()).collect::<Vec<_>>();
        unsafe {
            ffi::rubberband_study(
                self.as_ptr(),
                input.as_ptr(),
                num_samples.try_into().unwrap(),
                is_final.into(),
            );
        }
    }

    /// Provide a block of sample frames for processing.
    ///
    /// See also [`Self::get_samples_required`] and [`Self::set_max_process_size`].
    ///
    /// `input` should point to de-interleaved audio data with one float array per channel. Sample
    /// values are conventionally expected to be in the range -1.0f to +1.0f.
    ///
    /// Set `is_final` to true if this is the last block of input data.
    pub fn process(&self, input: &[&[f32]], is_final: bool) {
        let num_samples = input.iter().map(|s| s.len()).min().unwrap_or(0);
        let input = input.iter().map(|slice| slice.as_ptr()).collect::<Vec<_>>();
        unsafe {
            ffi::rubberband_process(
                self.as_ptr(),
                input.as_ptr(),
                num_samples.try_into().unwrap(),
                is_final.into(),
            );
        }
    }

    /// Ask the stretcher how many audio sample frames of output data are available for reading (via
    /// [`Self::retrieve`]).
    ///
    /// This function returns 0 if no frames are available: this usually means more input data needs
    /// to be provided, but if the stretcher is running in threaded mode it may just mean that not
    /// enough data has yet been processed. Call [`Self::get_samples_required`] to discover whether
    /// more input is needed.
    ///
    /// Note that the return value refers to the number of audio sample frames, which may be
    /// multi-channel, not the number of individual samples. (For example, one second of stereo
    /// audio sampled at 44100Hz yields a value of 44100 sample frames, not 88200.) This rule
    /// applies throughout the Rubber Band API.
    ///
    /// This function returns [`Option::None`] if all data has been fully processed and all output
    /// read, and the stretch process is now finished.
    pub fn available(&self) -> Option<u32> {
        unsafe { ffi::rubberband_available(self.as_ptr()) }
            .try_into()
            .ok()
    }

    /// Obtain some processed output data from the stretcher.
    ///
    /// Samples will be stored in each of the output arrays (one per channel for de-interleaved
    /// audio data) pointed to by `output`. The number of sample frames available to be retrieved
    /// can be queried beforehand with a call to [`Self::available`]. The return value is the actual
    /// number of sample frames retrieved.
    ///
    /// Note that the return value refer to the number of audio sample frames, which may be
    /// multi-channel, not the number of individual samples. (For example, one second of stereo
    /// audio sampled at 44100Hz yields a value of 44100 sample frames, not 88200.) This rule
    /// applies throughout the Rubber Band API.
    pub fn retrieve(&self, output: &mut [&mut [f32]]) -> u32 {
        let num_samples = output.iter().map(|s| s.len()).min().unwrap_or(0);
        let output = output
            .iter_mut()
            .map(|slice| slice.as_mut_ptr())
            .collect::<Vec<_>>();
        unsafe {
            ffi::rubberband_retrieve(
                self.as_ptr(),
                output.as_ptr(),
                num_samples.try_into().unwrap(),
            )
        }
    }

    /// Force the stretcher to calculate a stretch profile.
    ///
    /// Normally this happens automatically for the first [`Self::process`] call in offline mode.
    ///
    /// This function is provided for diagnostic purposes only and is supported only with the R2
    /// engine.
    pub fn calculate_stretch(&self) {
        unsafe {
            ffi::rubberband_calculate_stretch(self.as_ptr());
        }
    }

    /// Set the level of debug output.
    ///
    /// The default is whatever has been set using [`Self::set_default_debug_level`], or
    /// [`DebugLevel::None`] if that function has not been called.
    ///
    /// All output goes to cerr unless a custom logger has been provided on construction. Because
    /// writing to cerr is not RT-safe, only [`DebugLevel::None`] is RT-safe in normal use by
    /// default. Debug levels [`DebugLevel::None`] and [`DebugLevel::Error`] use only C-string
    /// constants as debug messages, so they are RT-safe if your custom logger is RT-safe.
    /// Levels [`DebugLevel::Info`] and [`DebugLevel::Verbose`] are not guaranteed to be RT-safe in
    /// any conditions as they may construct messages by allocation.
    pub fn set_debug_level(&self, level: DebugLevel) {
        unsafe {
            ffi::rubberband_set_debug_level(self.as_ptr(), level as i32);
        }
    }

    /// Set the default level of debug output for subsequently constructed stretchers.
    pub fn set_default_debug_level(level: DebugLevel) {
        unsafe {
            ffi::rubberband_set_default_debug_level(level as i32);
        }
    }
}

impl Drop for Stretcher {
    fn drop(&mut self) {
        unsafe { ffi::rubberband_delete(self.as_ptr()) }
    }
}

unsafe impl Send for Stretcher {}
