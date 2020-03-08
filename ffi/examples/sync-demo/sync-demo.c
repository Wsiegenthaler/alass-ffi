//
// sync-demo
//
// This demo cli utility demonstrates basic use of the alass-ffi bindings. See
// the generated 'alass.h' header for api details.
//

#include <stdio.h>
#include <string.h>
#include <libgen.h>
#include <getopt.h>

#include "alass.h"

const double NAN = 0.0/0.0;

//
// Generates an AlassTimeSpans instance by analyzing raw audio data for voice
// activity. This is for demonstration purposes but normally the audio data is
// extracted and resampled from a video file using a third-party library such
// as ffmpeg before being fed to alass-ffi.
//
// The resulting sample data fed into the AlassAudioSink must be 8000Hz mono
// 16-bit signed little-endian. Deallocation of AlassTimeSpans is the
// responsibility of the caller.
// 
AlassTimeSpans *load_audio_ref_spans(char *ref_file)
{
    FILE *file;
    int32_t buf_len = 4096;
    
    // Allocate buffer
    uint8_t *buffer = malloc(buf_len);
    if (!buffer)
    {
        fprintf(stderr, "Memory error!");
        return NULL;
    }
    
    // Create sink
    AlassAudioSink *sink = alass_audio_sink_new();
    if (!sink)
    {
        fprintf(stderr, "Unable to create AlassAudioSink!");
        free(buffer);
        return NULL;
    }
    
    // Open file containing raw samples
    file = fopen(ref_file, "rb");
    if (!file)
    {
        fprintf(stderr, "Unable to open reference audio file %s", ref_file);
        alass_audio_sink_free(sink);
        free(buffer);
        return NULL;
    }
    
    // Get file length
    fseek(file, 0, SEEK_END);
    int64_t byte_cnt = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    // Feed file contents to an AlassAudioSink
    while (byte_cnt > 0)
    {
        int64_t chunk_size = byte_cnt > buf_len ? buf_len : byte_cnt;
        fread(buffer, chunk_size, 1, file);
        alass_audio_sink_send(sink, buffer, chunk_size / 2);
        byte_cnt -= chunk_size;
    }
    fclose(file);
    free(buffer);
    
    // Compute voice activity
    AlassVoiceActivity *voice = alass_voice_activity_compute(sink);
    alass_audio_sink_free(sink);
    if (voice == NULL)
    {
        fprintf(stderr, "Unable to compute voice activity from reference audio file %s", ref_file);
        return NULL;
    }
    
    // Compute reference timespans from voice activity
    AlassTimeSpans *ref_spans = alass_timespans_compute(voice);
    alass_voice_activity_free(voice);
    if (ref_spans == NULL)
    {
        fprintf(stderr, "Unable to compute reference timespans from voice activity!");
        return NULL;
    }
    
    return ref_spans;
}

//
// Synchronizes subtitle file at 'sub_in' using the raw audio samples in 'ref_file'. The
// resulting output is saved to 'sub_out'.
//
AlassResultCode sync_to_audio(char *sub_in, char *sub_out, char *ref_file, float ref_fps, char *sub_enc, AlassSyncOptions *opts)
{
    // Open reference audio file and compute timespans
    AlassTimeSpans *ref_spans = load_audio_ref_spans(ref_file);
    if (ref_spans == NULL)
    {
      return 1;
    }

    // Synchronize
    AlassResultCode rc = alass_sync(sub_in, sub_out, ref_spans, ref_fps, sub_enc, opts);
    if (rc != ALASS_SUCCESS)
    {
      fprintf(stderr, "ERROR: Unable to synchronize subtitles!\n");
    }

    alass_timespans_free(ref_spans);

    return rc;
}

//
// Synchronizes subtitle file at 'sub_in' using the reference subtitle file at 'ref_file'. The
// resulting output is saved to 'sub_out'.
//
AlassResultCode sync_to_subtitle(char *sub_in, char *sub_out, char *ref_file, float ref_fps, char *sub_enc, char *ref_sub_enc, AlassSyncOptions *opts)
{
    // Open reference subtitle and generate timespans
    AlassTimeSpans *ref_spans = alass_timespans_load_subtitle(ref_file, ref_sub_enc);
    if (ref_spans == NULL)
    {
      fprintf(stderr, "ERROR: Unable to open reference subtitle file!\n");
      return 1;
    }

    // Synchronize
    AlassResultCode rc = alass_sync(sub_in, sub_out, ref_spans, ref_fps, sub_enc, opts);
    if (rc != ALASS_SUCCESS)
    {
      fprintf(stderr, "ERROR: Unable to synchronize subtitles!\n");
    }

    alass_timespans_free(ref_spans);

    return rc;
}

void usage(char **argv)
{
    char *cmd_name = basename(argv[0]);
    fprintf(stderr, "USAGE\n");
    fprintf(stderr, "  %s -s SUB_REF_FILE SUB_IN SUB_OUT\n", cmd_name);
    fprintf(stderr, "  %s -a PCM_REF_FILE SUB_IN SUB_OUT\n", cmd_name);
    fprintf(stderr, "\nARGUMENTS\n");
    fprintf(stderr, "  SUB_IN    Subtitle file with incorrect timing.\n");
    fprintf(stderr, "  SUB_OUT   Output location of fixed subtitle file.\n");
    fprintf(stderr, "\nOPTIONS\n");
    fprintf(stderr, "  -s, --ref-sub REF_SUB_FILE    Correctly-timed reference subtitle file to which to sync.\n");
    fprintf(stderr, "  -a, --ref-audio REF_PCM_FILE  Reference audio file to which to sync (raw 8kHz mono 16bit signed little-endian).\n");
    fprintf(stderr, "  -n, --no-split                Disable alass \"split mode\".\n");
    fprintf(stderr, "  -p, --split-penalty FLOAT     The penalty applied to each split when using \"split mode\". (default 7.0)\n");
    fprintf(stderr, "  -i, --interval MILLIS         Smallest recognized time interval by alass.\n");
    fprintf(stderr, "  -o, --optimization FLOAT      Higher values sacrifice accuracy for speed. (default 1.0, 0.0 to disable)\n");
    fprintf(stderr, "  -f, --ref-fps REF_FPS         Enables framerate correction and provides alass with the known fps of the reference file.\n");
    fprintf(stderr, "  -e, --sub-enc LABEL           IANA label of the subtitle charset (default: 'auto').\n");
    fprintf(stderr, "  -r, --ref-sub-enc LABEL       When using -s, the IANA label of the reference subtitle charset (default: 'auto').\n");
    fprintf(stderr, "  -v, --verbose\n");
    fprintf(stderr, "\n");
}

// Parses string to int64, returning min value when invalid
int64_t parse_int(char *str)
{
    char *end;
    double val = strtol(str, &end, 10);
    if (end != str + strlen(str)) return INT64_MIN;
    return val;
}

// Parses string to double, returning NAN when invalid
double parse_double(char *str)
{
    char *end;
    double val = strtod(str, &end);
    if (end != str + strlen(str)) return NAN;
    return val;
}

bool valid_int(int64_t val)
{
  return val != INT64_MIN;
}

bool valid_double(double val)
{
  return !(val != val); // !nan
}

int main (int argc, char **argv)
{
    char *sub_ref = NULL;
    char *aud_ref = NULL;
    char *sub_enc = NULL;
    char *ref_sub_enc = NULL;
    float ref_fps = NAN;
    bool verbose = false;
    
    AlassSyncOptions *sync_opts = alass_options_new();

    static struct option opts[] =
    {
        {"ref-sub",        required_argument, 0, 's'},
        {"ref-audio",      required_argument, 0, 'a'},
        {"sub-enc",        required_argument, 0, 'e'},
        {"ref-sub-enc",    required_argument, 0, 'r'},
        {"no-split",       no_argument,       0, 'n'},
        {"split-penalty",  required_argument, 0, 'p'},
        {"optimization",   required_argument, 0, 'o'},
        {"interval",       required_argument, 0, 'i'},
        {"ref-fps",        required_argument, 0, 'f'},
        {"verbose",        no_argument,       0, 'v'},
        {0, 0, 0, 0}
    };
    int opt_index = 0;
    int c;

    // Process cli args
    while ((c = getopt_long(argc, argv, "s:a:ne:r:i:f:p:o:v", opts, &opt_index)) != -1)
    {
        int64_t int_val;
        double dbl_val;

        switch (c)
        {
            case 's':
                sub_ref = optarg;
                break;
            case 'a':
                aud_ref = optarg;
                break;
            case 'e':
                sub_enc = optarg;
                break;
            case 'r':
                ref_sub_enc = optarg;
                break;
            case 'n':
                alass_options_set_split_mode(sync_opts, optarg);
                break;
            case 'i':
                int_val = parse_int(optarg);
                if (int_val <= 0)
                {
                    fprintf(stderr, "ERROR: Interval value must be a positive integer!\n");
                    return 1;
                }
                alass_options_set_interval(sync_opts, int_val);
                break;
            case 'f':
                ref_fps = parse_double(optarg);
                if (!valid_double(ref_fps))
                {
                    fprintf(stderr, "ERROR: Reference framerate param must be a valid float!\n");
                    return 1;
                } else if (ref_fps < 24.0 || ref_fps > 60.0)
                {
                    fprintf(stderr, "WARNING: Reference framerate param %.3f appears to be non-standard!\n", ref_fps);
                }
                break;
            case 'p':
                dbl_val = parse_double(optarg);
                if(!valid_double(dbl_val) || dbl_val <= 0.0 || dbl_val > 1000.0)
                {
                    fprintf(stderr, "ERROR: Split penalty param must be a valid float between 0 and 1000!\n");
                    return 1;
                }
                alass_options_set_split_penalty(sync_opts, dbl_val);
                break;
            case 'o':
                dbl_val = parse_double(optarg);
                if (!valid_double(dbl_val) || dbl_val < 0.0)
                {
                    fprintf(stderr, "ERROR: Speed optimization param must be a valid float greater than or equal to zero!\n");
                    return 1;
                }
                alass_options_set_speed_optimization(sync_opts, dbl_val);
                break;
            case 'v':
                verbose = true;
                break;
            default:
                usage(argv);
                return 1;
        }
    }

    // Obtain sub_in and sub_out args
    if (argc - optind != 2)
    {
        usage(argv);
        return 1;
    }
    char *sub_in = argv[optind];
    char *sub_out = argv[optind+1];
   
    // Ensure at least one ref_file is specified
    if ((aud_ref == NULL && sub_ref == NULL) || (aud_ref != NULL && sub_ref != NULL))
    {
        usage(argv);
        return 1;
    }
 
    // Configure logging (optional)
    AlassLogLevel log_level = verbose ? ALASS_LOG_TRACE : ALASS_LOG_WARN;
    alass_log_config(log_level, ALASS_LOG_ERROR, ALASS_LOG_NONE, NULL);
 
    // Print cli args
    printf(" [ sub-in      ] = %s\n", sub_in);
    printf(" [ sub-out     ] = %s\n", sub_out);
    if (sub_ref != NULL) printf(" [ ref-file    ] = %s\n", sub_ref);
    if (aud_ref != NULL) printf(" [ ref-file    ] = %s\n", aud_ref);
    if (sub_enc) printf(" [ sub-enc     ] = %s\n", sub_enc);
    if (ref_sub_enc) printf(" [ ref-sub-enc ] = %s\n", ref_sub_enc);
    alass_options_log(sync_opts);

    // Sync using either reference subtitle or pcm audio file
    AlassResultCode rc =
        sub_ref ?
            sync_to_subtitle(sub_in, sub_out, sub_ref, ref_fps, sub_enc, ref_sub_enc, sync_opts) :
            sync_to_audio(sub_in, sub_out, aud_ref, ref_fps, sub_enc, sync_opts);

    if (rc == ALASS_SUCCESS) printf("Sync complete.\n");

    alass_options_free(sync_opts);

    return rc;
}
